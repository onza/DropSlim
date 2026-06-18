use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use tauri::AppHandle;
use tokio::task::JoinSet;

use super::collect::{self, SUPPORTED_FORMATS_LABEL};
use super::events::{app_event_sink, EventSink, ProcessorEvent};
use super::image::optimize_image_file;
use super::output_path::{build_output_path, custom_save_folder_missing, UserSettings};
use super::summary::{build_batch_summary, build_optimize_summary, file_size};

const OPTIMIZATION_CONCURRENCY: usize = 3;
pub const MAX_BATCH_FILES: usize = 10_000;

struct BatchState {
    total: usize,
    pending: usize,
    succeeded: usize,
    failed: usize,
    bytes_before: u64,
    bytes_after: u64,
}

impl BatchState {
    fn new(total: usize) -> Self {
        Self {
            total,
            pending: total,
            succeeded: 0,
            failed: 0,
            bytes_before: 0,
            bytes_after: 0,
        }
    }

    fn done(&self) -> u32 {
        (self.succeeded + self.failed) as u32
    }

    fn complete_success(&mut self, bytes_before: u64, bytes_after: u64) {
        self.pending -= 1;
        self.succeeded += 1;
        self.bytes_before += bytes_before;
        self.bytes_after += bytes_after;
    }

    fn complete_failure(&mut self) {
        self.pending -= 1;
        self.failed += 1;
    }

    fn emit_progress<E: EventSink + ?Sized>(&self, sink: &E) {
        sink.send(ProcessorEvent::BatchProgress(
            super::events::BatchProgress {
                done: self.done(),
                total: self.total as u32,
            },
        ));
    }

    fn finish_if_done<E: EventSink + ?Sized>(&self, sink: &E) {
        if self.pending != 0 {
            return;
        }

        sink.send(ProcessorEvent::BatchComplete(build_batch_summary(
            self.total,
            self.succeeded,
            self.failed,
            self.bytes_before,
            self.bytes_after,
        )));
    }
}

fn drop_error(file_name: impl Into<String>, message: impl Into<String>) -> ProcessorEvent {
    ProcessorEvent::DropError {
        file_name: file_name.into(),
        message: message.into(),
    }
}

pub(crate) fn unsupported_selection_label(input_paths: &[String]) -> String {
    if input_paths.len() == 1 {
        PathBuf::from(&input_paths[0])
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Dropped selection")
            .to_string()
    } else {
        "Dropped selection".to_string()
    }
}

async fn process_file<E: EventSink + ?Sized + 'static>(
    sink: Arc<E>,
    file_path: PathBuf,
    settings: UserSettings,
    project_root: PathBuf,
    batch: Arc<tokio::sync::Mutex<BatchState>>,
) {
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image")
        .to_string();

    sink.send(ProcessorEvent::FileProcessing(file_name.clone()));

    let error_file_name = file_name.clone();

    let result = tokio::task::spawn_blocking(move || {
        let size_orig = file_size(&file_path).map_err(|error| error.to_string())?;
        let output_path =
            build_output_path(&file_path, &settings).map_err(|error| error.to_string())?;

        let previous_output_size = file_size(&output_path).ok();

        optimize_image_file(&file_path, &output_path, &project_root)?;

        let size_optimized = file_size(&output_path).map_err(|error| error.to_string())?;
        let summary = build_optimize_summary(size_orig, size_optimized, previous_output_size);

        Ok::<_, String>((output_path, summary, file_name, size_orig, size_optimized))
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|inner| inner);

    let mut batch = batch.lock().await;

    match result {
        Ok((output_path, summary, source_name, size_orig, size_optimized)) => {
            sink.send(ProcessorEvent::ImageOptimized {
                output_path: output_path.to_string_lossy().to_string(),
                summary,
                source_name,
            });
            batch.complete_success(size_orig, size_optimized);
        }
        Err(message) => {
            sink.send(drop_error(error_file_name, message));
            batch.complete_failure();
        }
    }

    batch.emit_progress(sink.as_ref());
    batch.finish_if_done(sink.as_ref());
}

pub async fn process_paths_with_sink<E: EventSink + ?Sized + 'static>(
    sink: Arc<E>,
    input_paths: Vec<String>,
    settings: UserSettings,
    project_root: PathBuf,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    if input_paths.is_empty() {
        return Ok(());
    }

    let collected =
        collect::collect_image_paths(&input_paths).map_err(|error| error.to_string())?;

    for missing in &collected.missing {
        sink.send(drop_error(
            missing.clone(),
            "File or folder not found.".to_string(),
        ));
    }

    let image_paths = collected.paths;
    let had_missing = !collected.missing.is_empty();

    if image_paths.is_empty() {
        if !had_missing {
            sink.send(drop_error(
                unsupported_selection_label(&input_paths),
                format!("No supported images found ({SUPPORTED_FORMATS_LABEL})."),
            ));
        }
        return Ok(());
    }

    if image_paths.len() > MAX_BATCH_FILES {
        sink.send(drop_error(
            "Batch",
            format!(
                "Too many images ({}). Maximum is {MAX_BATCH_FILES} per batch.",
                image_paths.len()
            ),
        ));
        return Ok(());
    }

    if custom_save_folder_missing(&settings) {
        sink.send(drop_error(
            "Settings",
            "Please choose a save folder in Settings first.",
        ));
        return Ok(());
    }

    let total = image_paths.len();
    sink.send(ProcessorEvent::BatchStarted {
        total: total as u32,
    });

    let paths = Arc::new(image_paths);
    let batch = Arc::new(tokio::sync::Mutex::new(BatchState::new(total)));
    let next_index = Arc::new(AtomicUsize::new(0));
    let worker_count = OPTIMIZATION_CONCURRENCY.min(total);
    let mut tasks = JoinSet::new();

    for _ in 0..worker_count {
        let sink = Arc::clone(&sink);
        let settings = settings.clone();
        let project_root = project_root.clone();
        let batch = Arc::clone(&batch);
        let paths = Arc::clone(&paths);
        let next_index = Arc::clone(&next_index);
        let cancel = Arc::clone(&cancel);

        tasks.spawn(async move {
            loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }

                let index = next_index.fetch_add(1, Ordering::Relaxed);
                if index >= paths.len() {
                    break;
                }

                process_file(
                    Arc::clone(&sink),
                    paths[index].clone(),
                    settings.clone(),
                    project_root.clone(),
                    Arc::clone(&batch),
                )
                .await;
            }
        });
    }

    while tasks.join_next().await.is_some() {}

    if cancel.load(Ordering::Relaxed) {
        let batch = batch.lock().await;
        if batch.pending > 0 {
            sink.send(ProcessorEvent::BatchCancelled {
                done: batch.done(),
                total: total as u32,
                succeeded: batch.succeeded as u32,
                failed: batch.failed as u32,
            });
        }
    }

    Ok(())
}

pub async fn process_paths(
    app: AppHandle,
    input_paths: Vec<String>,
    settings: UserSettings,
    project_root: PathBuf,
    cancel: Arc<AtomicBool>,
) -> Result<(), String> {
    process_paths_with_sink(
        app_event_sink(app),
        input_paths,
        settings,
        project_root,
        cancel,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::events::RecordingEventSink;
    use image::{ImageBuffer, Rgba};
    use std::fs;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn no_cancel() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    fn write_png(path: &std::path::Path) {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(320, 240, |x, y| Rgba([x as u8, y as u8, 128, 255]));
        img.save(path).expect("write png");
    }

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn batch_summary_only_after_multi_file_batch() {
        let mut batch = BatchState::new(2);
        batch.complete_success(100, 80);
        assert_eq!(batch.pending, 1);

        batch.complete_failure();
        assert_eq!(batch.pending, 0);
        assert_eq!(batch.succeeded, 1);
        assert_eq!(batch.failed, 1);
    }

    #[test]
    fn single_file_batch_emits_summary_when_done() {
        let recording = RecordingEventSink::new();
        let mut batch = BatchState::new(1);
        batch.complete_success(100_000, 40_000);
        batch.finish_if_done(&recording);

        assert_eq!(
            recording.events(),
            vec![ProcessorEvent::BatchComplete(
                "1 image · saved 60 KB (60%)".to_string()
            )]
        );
    }

    #[test]
    fn unsupported_selection_label_uses_file_name() {
        let label = unsupported_selection_label(&["/tmp/notes.txt".to_string()]);
        assert_eq!(label, "notes.txt");
    }

    #[tokio::test]
    async fn empty_paths_emit_nothing() {
        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![],
            UserSettings::default(),
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        assert!(recording.events().is_empty());
    }

    #[tokio::test]
    async fn missing_paths_emit_drop_error() {
        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec!["/no/such/photo.png".to_string()],
            UserSettings::default(),
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        assert_eq!(
            recording.events(),
            vec![ProcessorEvent::DropError {
                file_name: "photo.png".to_string(),
                message: "File or folder not found.".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn unsupported_files_emit_drop_error() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("notes.txt");
        fs::write(&file, b"text").expect("write text");

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings::default(),
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        assert_eq!(
            recording.events(),
            vec![ProcessorEvent::DropError {
                file_name: "notes.txt".to_string(),
                message: format!("No supported images found ({SUPPORTED_FORMATS_LABEL})."),
            }]
        );
    }

    #[tokio::test]
    async fn empty_save_folder_emits_settings_error() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("photo.png");
        write_png(&file);

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings {
                folderswitch: false,
                savepath: Some(vec![]),
                ..Default::default()
            },
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        assert_eq!(
            recording.events(),
            vec![ProcessorEvent::DropError {
                file_name: "Settings".to_string(),
                message: "Please choose a save folder in Settings first.".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn missing_save_folder_emits_settings_error() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("photo.png");
        write_png(&file);

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings {
                folderswitch: false,
                savepath: None,
                ..Default::default()
            },
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        assert_eq!(
            recording.events(),
            vec![ProcessorEvent::DropError {
                file_name: "Settings".to_string(),
                message: "Please choose a save folder in Settings first.".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn single_image_emits_processing_optimized_and_batch_summary() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("photo.png");
        write_png(&file);

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings::default(),
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        let events = recording.events();
        assert!(events
            .iter()
            .any(|event| { matches!(event, ProcessorEvent::BatchStarted { total: 1 }) }));
        assert!(matches!(
            events.iter().find(|event| matches!(event, ProcessorEvent::FileProcessing(_))),
            Some(ProcessorEvent::FileProcessing(ref name)) if name == "photo.png"
        ));
        assert!(events
            .iter()
            .any(|event| matches!(event, ProcessorEvent::ImageOptimized { .. })));
        assert!(events
            .iter()
            .any(|event| matches!(event, ProcessorEvent::BatchComplete(_))));
    }

    #[tokio::test]
    async fn batch_emits_summary_after_multiple_images() {
        let dir = tempdir().expect("tempdir");
        let first = dir.path().join("a.png");
        let second = dir.path().join("b.png");
        write_png(&first);
        write_png(&second);

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![
                first.to_string_lossy().to_string(),
                second.to_string_lossy().to_string(),
            ],
            UserSettings::default(),
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        let events = recording.events();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, ProcessorEvent::FileProcessing(_)))
                .count(),
            2
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, ProcessorEvent::ImageOptimized { .. }))
                .count(),
            2
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, ProcessorEvent::BatchComplete(_)))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn cancel_stops_remaining_files() {
        let dir = tempdir().expect("tempdir");
        let paths: Vec<String> = (0..6)
            .map(|index| {
                let file = dir.path().join(format!("photo-{index}.png"));
                write_png(&file);
                file.to_string_lossy().to_string()
            })
            .collect();

        let cancel = Arc::new(AtomicBool::new(false));
        let recording = Arc::new(RecordingEventSink::new());
        let sink = Arc::clone(&recording);
        let cancel_flag = Arc::clone(&cancel);
        let paths_clone = paths.clone();

        let handle = tokio::spawn(async move {
            process_paths_with_sink(
                sink,
                paths_clone,
                UserSettings::default(),
                project_root(),
                cancel_flag,
            )
            .await
            .expect("process paths");
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        cancel.store(true, Ordering::SeqCst);
        handle.await.expect("join batch");

        let events = recording.events();
        let optimized = events
            .iter()
            .filter(|event| matches!(event, ProcessorEvent::ImageOptimized { .. }))
            .count();
        assert!(optimized < 6);
        assert!(events
            .iter()
            .any(|event| matches!(event, ProcessorEvent::BatchCancelled { total: 6, .. })));
    }
}
