#[cfg(test)]
use std::sync::{Arc, Mutex};

use super::image::DimensionChange;
use super::payloads::{BatchSummaryPayload, ErrorPayload, SummaryPayload};

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
        summary: SummaryPayload,
        source_name: String,
        resized: Option<DimensionChange>,
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

pub trait EventSink: Send + Sync {
    fn send(&self, event: ProcessorEvent);
}

#[cfg(test)]
#[derive(Clone, Default)]
pub struct RecordingEventSink {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payloads::{BatchSummaryPayload, ErrorPayload, SummaryPayload};

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
