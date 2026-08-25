//! Image optimization core shared by the DropSlim app and CLI.

mod animation;
mod collect;
mod events;
pub mod formats;
mod heic;
mod image;
mod output_path;
mod payloads;
mod processor;
mod summary;
mod temp_paths;
mod tools;

pub use events::{BatchProgress, EventSink, ProcessorEvent};
pub use formats::OutputFormatSetting;
pub use image::{optimize_image_file, DimensionChange, DimensionLimits};
pub use output_path::UserSettings;
pub use payloads::{BatchSummaryPayload, ErrorPayload, SummaryPayload};
pub use processor::{process_paths_with_sink, MAX_BATCH_FILES};

#[cfg(test)]
pub use events::RecordingEventSink;
