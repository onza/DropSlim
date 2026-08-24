use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::task::JoinSet;

use super::collect;
use super::events::{EventSink, ProcessorEvent};
use super::formats::{ImageFormat, OutputFormatSetting};
use super::image::{optimize_image_file, DimensionChange};
use super::output_path::{build_output_path, custom_save_folder_missing, UserSettings};
use super::payloads::ErrorPayload;
use super::summary::{
    build_batch_summary_payload, build_optimize_summary_payload, file_size,
    should_keep_optimized_output,
};
use super::temp_paths::TempFile;

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

        sink.send(ProcessorEvent::BatchComplete(build_batch_summary_payload(
            self.total,
            self.succeeded,
            self.failed,
            self.bytes_before,
            self.bytes_after,
        )));
    }
}

fn drop_error(file_name: impl Into<String>, error: ErrorPayload) -> ProcessorEvent {
    ProcessorEvent::DropError {
        file_name: file_name.into(),
        error,
    }
}

fn commit_candidate(
    candidate: &std::path::Path,
    output: &std::path::Path,
) -> Result<(), ErrorPayload> {
    if std::fs::rename(candidate, output).is_err() {
        std::fs::copy(candidate, output).map_err(|error| ErrorPayload::io(error.to_string()))?;
        std::fs::remove_file(candidate).map_err(|error| ErrorPayload::io(error.to_string()))?;
    }

    Ok(())
}

fn report_output_path(
    file_path: &std::path::Path,
    output_path: &std::path::Path,
    previous_output_size: Option<u64>,
    kept_output: bool,
) -> PathBuf {
    if kept_output {
        return output_path.to_path_buf();
    }

    if previous_output_size.is_some() {
        output_path.to_path_buf()
    } else {
        file_path.to_path_buf()
    }
}

fn is_real_format_convert(file_path: &std::path::Path, output_format: OutputFormatSetting) -> bool {
    let Some(source) = ImageFormat::from_path(file_path) else {
        return false;
    };

    if matches!(source, ImageFormat::Gif | ImageFormat::Svg) {
        return false;
    }

    match output_format.target_format() {
        Some(target) => target != source,
        None => false,
    }
}

struct ResolvedOptimization {
    output_path: PathBuf,
    summary: super::payloads::SummaryPayload,
    size_after: u64,
    resized: Option<DimensionChange>,
}

fn resolve_optimization(
    file_path: &std::path::Path,
    output_path: &std::path::Path,
    project_root: &std::path::Path,
    dimension_limits: Option<super::image::DimensionLimits>,
    output_format: OutputFormatSetting,
) -> Result<ResolvedOptimization, ErrorPayload> {
    let size_orig = file_size(file_path).map_err(|error| ErrorPayload::io(error.to_string()))?;
    let previous_output_size = file_size(output_path).ok();
    let threshold = previous_output_size.unwrap_or(size_orig);
    let candidate = TempFile::at(output_path);
    let candidate_path = candidate.path();
    let force_keep = is_real_format_convert(file_path, output_format);

    let resized = optimize_image_file(
        file_path,
        candidate_path,
        project_root,
        dimension_limits,
        output_format,
    )?;

    let candidate_size =
        file_size(candidate_path).map_err(|error| ErrorPayload::io(error.to_string()))?;

    let keep =
        force_keep || should_keep_optimized_output(candidate_size, size_orig, previous_output_size);

    if keep {
        commit_candidate(candidate_path, output_path)?;
        Ok(ResolvedOptimization {
            output_path: output_path.to_path_buf(),
            summary: build_optimize_summary_payload(
                size_orig,
                candidate_size,
                previous_output_size,
            ),
            size_after: candidate_size,
            resized,
        })
    } else {
        Ok(ResolvedOptimization {
            output_path: report_output_path(file_path, output_path, previous_output_size, false),
            summary: build_optimize_summary_payload(size_orig, threshold, previous_output_size),
            size_after: threshold,
            resized: None,
        })
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
        let output_path = build_output_path(&file_path, &settings)
            .map_err(|error| ErrorPayload::io(error.to_string()))?;
        let size_orig =
            file_size(&file_path).map_err(|error| ErrorPayload::io(error.to_string()))?;
        let resolved = resolve_optimization(
            &file_path,
            &output_path,
            &project_root,
            settings.dimension_limits(),
            settings.output_format,
        )?;

        Ok::<_, ErrorPayload>((
            resolved.output_path,
            resolved.summary,
            file_name,
            size_orig,
            resolved.size_after,
            resolved.resized,
        ))
    })
    .await
    .map_err(|error| ErrorPayload::io(error.to_string()))
    .and_then(|inner| inner);

    let mut batch = batch.lock().await;

    match result {
        Ok((output_path, summary, source_name, size_orig, size_optimized, resized)) => {
            sink.send(ProcessorEvent::ImageOptimized {
                output_path: output_path.to_string_lossy().to_string(),
                summary,
                source_name,
                resized,
            });
            batch.complete_success(size_orig, size_optimized);
        }
        Err(error) => {
            sink.send(drop_error(error_file_name, error));
            batch.complete_failure();
        }
    }

    batch.emit_progress(sink.as_ref());
    batch.finish_if_done(sink.as_ref());
}

pub(crate) async fn process_paths_with_sink<E: EventSink + ?Sized + 'static>(
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
        sink.send(drop_error(missing.clone(), ErrorPayload::file_not_found()));
    }

    for unreadable in &collected.unreadable {
        sink.send(drop_error(
            unreadable.clone(),
            ErrorPayload::io("Could not read file path."),
        ));
    }

    let image_paths = collected.paths;
    let had_missing = !collected.missing.is_empty();
    let had_unreadable = !collected.unreadable.is_empty();

    if image_paths.is_empty() {
        if !had_missing && !had_unreadable {
            sink.send(drop_error(
                unsupported_selection_label(&input_paths),
                ErrorPayload::no_supported_images(),
            ));
        }
        return Ok(());
    }

    if image_paths.len() > MAX_BATCH_FILES {
        sink.send(drop_error(
            "Batch",
            ErrorPayload::too_many_images(image_paths.len() as u32, MAX_BATCH_FILES as u32),
        ));
        return Ok(());
    }

    if custom_save_folder_missing(&settings) {
        sink.send(drop_error("Settings", ErrorPayload::save_folder_required()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::events::RecordingEventSink;
    use crate::optimize::optimize_image_file;
    use crate::optimize::payloads::{BatchSummaryPayload, ErrorPayload, SummaryPayload};
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
            vec![ProcessorEvent::BatchComplete(BatchSummaryPayload {
                total: 1,
                succeeded: 1,
                failed: 0,
                bytes_before: 100_000,
                bytes_after: 40_000,
            })]
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
                error: ErrorPayload::file_not_found(),
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
                error: ErrorPayload::no_supported_images(),
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
                error: ErrorPayload::save_folder_required(),
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
                error: ErrorPayload::save_folder_required(),
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
    async fn reoptimization_does_not_grow_existing_output() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("photo.png");
        write_png(&file);
        let output = dir.path().join("photo.min.png");

        optimize_image_file(
            &file,
            &output,
            &project_root(),
            None,
            crate::optimize::formats::OutputFormatSetting::Original,
        )
        .expect("seed output");
        let first_size = fs::metadata(&output).expect("output metadata").len();

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings::default(),
            project_root(),
            no_cancel(),
        )
        .await
        .expect("second pass");

        let second_size = fs::metadata(&output).expect("output metadata").len();
        assert!(second_size <= first_size);

        let summary = recording
            .events()
            .into_iter()
            .find_map(|event| match event {
                ProcessorEvent::ImageOptimized { summary, .. } => Some(summary),
                _ => None,
            })
            .expect("optimized event");

        assert!(matches!(summary, SummaryPayload::AlreadyOptimized { .. }));
    }

    #[tokio::test]
    async fn skip_if_larger_does_not_create_side_by_side_output() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("tiny.png");
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        img.save(&file).expect("write tiny png");

        let output = dir.path().join("tiny.min.png");
        fs::write(&output, &[0]).expect("write tiny existing output");

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

        assert_eq!(fs::metadata(&output).expect("metadata").len(), 1);
        assert!(!dir.path().join("tiny.dropslim.png").exists());

        let summary = recording
            .events()
            .into_iter()
            .find_map(|event| match event {
                ProcessorEvent::ImageOptimized { summary, .. } => Some(summary),
                _ => None,
            })
            .expect("optimized event");

        assert!(matches!(summary, SummaryPayload::AlreadyOptimized { .. }));
    }

    #[test]
    fn detects_real_format_convert() {
        assert!(is_real_format_convert(
            std::path::Path::new("photo.png"),
            OutputFormatSetting::Avif
        ));
        assert!(is_real_format_convert(
            std::path::Path::new("photo.jpeg"),
            OutputFormatSetting::Webp
        ));
        assert!(!is_real_format_convert(
            std::path::Path::new("photo.png"),
            OutputFormatSetting::Original
        ));
        assert!(!is_real_format_convert(
            std::path::Path::new("photo.jpeg"),
            OutputFormatSetting::Jpeg
        ));
        assert!(!is_real_format_convert(
            std::path::Path::new("anim.gif"),
            OutputFormatSetting::Avif
        ));
    }

    #[tokio::test]
    async fn keeps_convert_output_even_when_larger_than_source() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("tiny.png");
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        img.save(&file).expect("write tiny png");
        let source_size = fs::metadata(&file).expect("source metadata").len();

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings {
                output_format: OutputFormatSetting::Avif,
                ..Default::default()
            },
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        let avif = dir.path().join("tiny.min.avif");
        assert!(
            avif.is_file(),
            "convert to avif must keep output even if larger than source ({source_size} bytes)"
        );
        let avif_size = fs::metadata(&avif).expect("avif metadata").len();
        assert!(avif_size > 12, "avif output should contain data");

        let output_path = recording
            .events()
            .into_iter()
            .find_map(|event| match event {
                ProcessorEvent::ImageOptimized { output_path, .. } => Some(output_path),
                _ => None,
            })
            .expect("optimized event");

        assert_eq!(PathBuf::from(&output_path).file_name(), avif.file_name());
        assert!(
            PathBuf::from(&output_path)
                .extension()
                .and_then(|e| e.to_str())
                == Some("avif")
        );
    }

    #[tokio::test]
    async fn emits_resized_when_dimension_limits_scale_image() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("large.png");
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(80, 60, Rgba([12, 34, 56, 255]));
        img.save(&file).expect("write png");

        let recording = Arc::new(RecordingEventSink::new());
        process_paths_with_sink(
            Arc::clone(&recording),
            vec![file.to_string_lossy().to_string()],
            UserSettings {
                limit_dimensions: true,
                max_width: Some(40),
                max_height: None,
                ..Default::default()
            },
            project_root(),
            no_cancel(),
        )
        .await
        .expect("process paths");

        let resized = recording
            .events()
            .into_iter()
            .find_map(|event| match event {
                ProcessorEvent::ImageOptimized { resized, .. } => resized,
                _ => None,
            })
            .expect("resized payload");

        assert_eq!(
            resized,
            DimensionChange {
                from_width: 80,
                from_height: 60,
                to_width: 40,
                to_height: 30,
            }
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
