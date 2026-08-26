use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;

use dropslim_core::{
    BatchProgress, DimensionChange, ErrorPayload, EventSink, ProcessorEvent, SummaryPayload,
};
use serde::Serialize;

pub struct CliEventSink {
    json: bool,
    quiet: bool,
    failed: AtomicU32,
    cancelled: AtomicBool,
    stderr: Mutex<()>,
}

impl CliEventSink {
    pub fn new(json: bool, quiet: bool) -> Self {
        Self {
            json,
            quiet,
            failed: AtomicU32::new(0),
            cancelled: AtomicBool::new(false),
            stderr: Mutex::new(()),
        }
    }

    pub fn failed(&self) -> u32 {
        self.failed.load(Ordering::SeqCst)
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    fn write_json(&self, value: &impl Serialize) {
        match serde_json::to_string(value) {
            Ok(line) => {
                let mut out = io::stdout().lock();
                let _ = writeln!(out, "{line}");
            }
            Err(error) => {
                let _guard = self.stderr.lock().ok();
                let _ = writeln!(io::stderr(), "json encode failed: {error}");
            }
        }
    }

    fn write_stderr(&self, message: &str) {
        let _guard = self.stderr.lock().ok();
        let _ = writeln!(io::stderr(), "{message}");
    }

    fn write_stdout(&self, message: &str) {
        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{message}");
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum JsonEvent<'a> {
    BatchStarted {
        total: u32,
    },
    ImageOptimized {
        output_path: &'a str,
        source_name: &'a str,
        summary: &'a SummaryPayload,
        #[serde(skip_serializing_if = "Option::is_none")]
        resized: Option<&'a DimensionChange>,
    },
    DropError {
        file_name: &'a str,
        error: &'a ErrorPayload,
    },
    BatchComplete {
        total: u32,
        succeeded: u32,
        failed: u32,
        bytes_before: u64,
        bytes_after: u64,
    },
    BatchCancelled {
        done: u32,
        total: u32,
        succeeded: u32,
        failed: u32,
    },
}

impl EventSink for CliEventSink {
    fn send(&self, event: ProcessorEvent) {
        match &event {
            ProcessorEvent::DropError { .. } => {
                self.failed.fetch_add(1, Ordering::SeqCst);
            }
            ProcessorEvent::BatchCancelled { .. } => {
                self.cancelled.store(true, Ordering::SeqCst);
            }
            _ => {}
        }

        if self.json {
            self.emit_json(&event);
        } else {
            self.emit_human(&event);
        }
    }
}

impl CliEventSink {
    fn emit_json(&self, event: &ProcessorEvent) {
        let json_event = match event {
            ProcessorEvent::BatchStarted { total } => {
                if self.quiet {
                    return;
                }
                JsonEvent::BatchStarted { total: *total }
            }
            ProcessorEvent::FileProcessing(file_name) => {
                if self.quiet {
                    return;
                }
                // progress stays on stderr in json mode
                self.write_stderr(&format!("processing {file_name}"));
                return;
            }
            ProcessorEvent::BatchProgress(BatchProgress { done, total }) => {
                if self.quiet {
                    return;
                }
                self.write_stderr(&format!("progress {done}/{total}"));
                return;
            }
            ProcessorEvent::ImageOptimized {
                output_path,
                summary,
                source_name,
                resized,
            } => {
                if self.quiet {
                    return;
                }
                JsonEvent::ImageOptimized {
                    output_path,
                    source_name,
                    summary,
                    resized: resized.as_ref(),
                }
            }
            ProcessorEvent::DropError { file_name, error } => JsonEvent::DropError {
                file_name,
                error,
            },
            ProcessorEvent::BatchComplete(summary) => JsonEvent::BatchComplete {
                total: summary.total,
                succeeded: summary.succeeded,
                failed: summary.failed,
                bytes_before: summary.bytes_before,
                bytes_after: summary.bytes_after,
            },
            ProcessorEvent::BatchCancelled {
                done,
                total,
                succeeded,
                failed,
            } => JsonEvent::BatchCancelled {
                done: *done,
                total: *total,
                succeeded: *succeeded,
                failed: *failed,
            },
        };

        self.write_json(&json_event);
    }

    fn emit_human(&self, event: &ProcessorEvent) {
        match event {
            ProcessorEvent::BatchStarted { total } => {
                if !self.quiet {
                    self.write_stderr(&format!("compressing {total} file(s)"));
                }
            }
            ProcessorEvent::FileProcessing(file_name) => {
                if !self.quiet {
                    self.write_stderr(&format!("… {file_name}"));
                }
            }
            ProcessorEvent::BatchProgress(BatchProgress { done, total }) => {
                if !self.quiet {
                    self.write_stderr(&format!("[{done}/{total}]"));
                }
            }
            ProcessorEvent::ImageOptimized {
                output_path,
                summary,
                source_name,
                resized,
            } => {
                if self.quiet {
                    return;
                }
                let resize = resized
                    .as_ref()
                    .map(|change| {
                        format!(
                            " ({}×{} → {}×{})",
                            change.from_width,
                            change.from_height,
                            change.to_width,
                            change.to_height
                        )
                    })
                    .unwrap_or_default();
                let detail = match summary {
                    SummaryPayload::AlreadyOptimized { size } => {
                        format!("already optimized ({size})")
                    }
                    SummaryPayload::Saved { percent, from, to }
                    | SummaryPayload::SavedMore { percent, from, to } => {
                        format!("-{percent}% ({from} → {to})")
                    }
                };
                self.write_stdout(&format!("{source_name} → {output_path}{resize} {detail}"));
            }
            ProcessorEvent::DropError { file_name, error } => {
                let detail = error
                    .detail
                    .as_deref()
                    .map(|value| format!(" ({value})"))
                    .unwrap_or_default();
                self.write_stderr(&format!("error: {file_name}: {}{detail}", error.code));
            }
            ProcessorEvent::BatchComplete(summary) => {
                if self.quiet {
                    return;
                }
                self.write_stderr(&format!(
                    "done: {} ok, {} failed ({} → {})",
                    summary.succeeded,
                    summary.failed,
                    summary.bytes_before,
                    summary.bytes_after
                ));
            }
            ProcessorEvent::BatchCancelled {
                done,
                total,
                succeeded,
                failed,
            } => {
                self.write_stderr(&format!(
                    "cancelled after {done}/{total} ({succeeded} ok, {failed} failed)"
                ));
            }
        }
    }
}
