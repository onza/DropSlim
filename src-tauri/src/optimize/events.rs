use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::payloads::{BatchSummaryPayload, ErrorPayload, SummaryPayload};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BatchProgress {
    pub done: u32,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessorEvent {
    BatchStarted {
        total: u32,
    },
    FileProcessing(String),
    BatchProgress(BatchProgress),
    ImageOptimized {
        output_path: String,
        summary: SummaryPayload,
        source_name: String,
        resized: Option<crate::optimize::DimensionChange>,
    },
    DropError {
        file_name: String,
        error: ErrorPayload,
    },
    BatchComplete(BatchSummaryPayload),
    BatchCancelled {
        done: u32,
        total: u32,
        succeeded: u32,
        failed: u32,
    },
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImageOptimizedEvent {
    output_path: String,
    summary: SummaryPayload,
    source_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resized: Option<crate::optimize::DimensionChange>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DropErrorEvent {
    file_name: String,
    error: ErrorPayload,
}

pub(crate) trait EventSink: Send + Sync {
    fn send(&self, event: ProcessorEvent);
}

struct AppEventSink(AppHandle);

impl EventSink for AppEventSink {
    fn send(&self, event: ProcessorEvent) {
        match event {
            ProcessorEvent::BatchStarted { total } => {
                if let Err(error) = self.0.emit("batch-started", total) {
                    eprintln!("batch-started emit failed: {error}");
                }
            }
            ProcessorEvent::FileProcessing(file_name) => {
                if let Err(error) = self.0.emit("file-processing", file_name) {
                    eprintln!("file-processing emit failed: {error}");
                }
            }
            ProcessorEvent::BatchProgress(progress) => {
                if let Err(error) = self
                    .0
                    .emit("batch-progress", (progress.done, progress.total))
                {
                    eprintln!("batch-progress emit failed: {error}");
                }
            }
            ProcessorEvent::ImageOptimized {
                output_path,
                summary,
                source_name,
                resized,
            } => {
                if let Err(error) = self.0.emit(
                    "image-optimized",
                    ImageOptimizedEvent {
                        output_path,
                        summary,
                        source_name,
                        resized,
                    },
                ) {
                    eprintln!("image-optimized emit failed: {error}");
                }
            }
            ProcessorEvent::DropError { file_name, error } => {
                if let Err(error) = self
                    .0
                    .emit("drop-error", DropErrorEvent { file_name, error })
                {
                    eprintln!("drop-error emit failed: {error}");
                }
            }
            ProcessorEvent::BatchComplete(summary) => {
                if let Err(error) = self.0.emit("batch-complete", summary) {
                    eprintln!("batch-complete emit failed: {error}");
                }
            }
            ProcessorEvent::BatchCancelled {
                done,
                total,
                succeeded,
                failed,
            } => {
                if let Err(error) = self
                    .0
                    .emit("batch-cancelled", (done, total, succeeded, failed))
                {
                    eprintln!("batch-cancelled emit failed: {error}");
                }
            }
        }
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct RecordingEventSink {
    events: Arc<Mutex<Vec<ProcessorEvent>>>,
}

#[cfg(test)]
impl RecordingEventSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<ProcessorEvent> {
        self.events.lock().expect("recording events lock").clone()
    }
}

#[cfg(test)]
impl EventSink for RecordingEventSink {
    fn send(&self, event: ProcessorEvent) {
        self.events
            .lock()
            .expect("recording events lock")
            .push(event);
    }
}

pub(crate) fn app_event_sink(app: AppHandle) -> Arc<dyn EventSink> {
    Arc::new(AppEventSink(app))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimize::payloads::{BatchSummaryPayload, ErrorPayload, SummaryPayload};

    #[test]
    fn records_batch_and_error_events() {
        let sink = RecordingEventSink::new();

        sink.send(ProcessorEvent::BatchStarted { total: 2 });
        sink.send(ProcessorEvent::DropError {
            file_name: "photo.png".to_string(),
            error: ErrorPayload::file_not_found(),
        });
        sink.send(ProcessorEvent::BatchComplete(BatchSummaryPayload {
            total: 1,
            succeeded: 1,
            failed: 0,
            bytes_before: 100,
            bytes_after: 40,
        }));

        assert_eq!(
            sink.events(),
            vec![
                ProcessorEvent::BatchStarted { total: 2 },
                ProcessorEvent::DropError {
                    file_name: "photo.png".to_string(),
                    error: ErrorPayload::file_not_found(),
                },
                ProcessorEvent::BatchComplete(BatchSummaryPayload {
                    total: 1,
                    succeeded: 1,
                    failed: 0,
                    bytes_before: 100,
                    bytes_after: 40,
                }),
            ]
        );
    }

    #[test]
    fn records_image_optimized_payload() {
        let sink = RecordingEventSink::new();

        sink.send(ProcessorEvent::ImageOptimized {
            output_path: "/tmp/photo.min.png".to_string(),
            summary: SummaryPayload::AlreadyOptimized {
                size: "40 KB".to_string(),
            },
            source_name: "photo.png".to_string(),
            resized: None,
        });

        assert!(matches!(
            sink.events().as_slice(),
            [ProcessorEvent::ImageOptimized { .. }]
        ));
    }
}
