use std::sync::Arc;

use dropslim_core::{
    BatchProgress, DimensionChange, ErrorPayload, EventSink, ProcessorEvent, SummaryPayload,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImageOptimizedEvent {
    output_path: String,
    summary: SummaryPayload,
    source_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    resized: Option<DimensionChange>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DropErrorEvent {
    file_name: String,
    error: ErrorPayload,
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
            ProcessorEvent::BatchProgress(BatchProgress { done, total }) => {
                if let Err(error) = self.0.emit("batch-progress", (done, total)) {
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

pub(crate) fn app_event_sink(app: AppHandle) -> Arc<dyn EventSink> {
    Arc::new(AppEventSink(app))
}
