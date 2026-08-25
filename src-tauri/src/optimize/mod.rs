//! Re-exports the shared optimization core and keeps the Tauri event adapter local.

pub use dropslim_core::formats;
pub use dropslim_core::{
    optimize_image_file, process_paths_with_sink, DimensionChange, DimensionLimits, ErrorPayload,
    EventSink, OutputFormatSetting, ProcessorEvent, UserSettings, MAX_BATCH_FILES,
};

pub(crate) use crate::optimize_events::app_event_sink;
