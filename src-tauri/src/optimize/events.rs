use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchProgress {
    pub done: u32,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessorEvent {
    BatchStarted {
        total: u32,
    },
    FileProcessing(String),
    BatchProgress(BatchProgress),
    ImageOptimized {
        output_path: String,
        summary: String,
        source_name: String,
    },
    DropError {
        file_name: String,
        message: String,
    },
    BatchComplete(String),
    BatchCancelled {
        done: u32,
        total: u32,
        succeeded: u32,
        failed: u32,
    },
}

pub trait EventSink: Send + Sync {
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
                if let Err(error) = self.0.emit("batch-progress", (progress.done, progress.total))
                {
                    eprintln!("batch-progress emit failed: {error}");
                }
            }
            ProcessorEvent::ImageOptimized {
                output_path,
                summary,
                source_name,
            } => {
                if let Err(error) = self.0.emit(
                    "image-optimized",
                    (output_path, summary, source_name),
                ) {
                    eprintln!("image-optimized emit failed: {error}");
                }
            }
            ProcessorEvent::DropError { file_name, message } => {
                if let Err(error) = self.0.emit("drop-error", (file_name, message)) {
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

#[derive(Clone, Default)]
pub struct RecordingEventSink {
    events: Arc<Mutex<Vec<ProcessorEvent>>>,
}

impl RecordingEventSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<ProcessorEvent> {
        self.events.lock().expect("recording events lock").clone()
    }
}

impl EventSink for RecordingEventSink {
    fn send(&self, event: ProcessorEvent) {
        self.events
            .lock()
            .expect("recording events lock")
            .push(event);
    }
}

pub fn app_event_sink(app: AppHandle) -> Arc<dyn EventSink> {
    Arc::new(AppEventSink(app))
}
