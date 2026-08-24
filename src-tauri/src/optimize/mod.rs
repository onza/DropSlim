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

pub use formats::OutputFormatSetting;
pub use image::{optimize_image_file, DimensionChange, DimensionLimits};
pub use output_path::UserSettings;
pub use payloads::ErrorPayload;
pub(crate) use events::app_event_sink;
pub(crate) use processor::process_paths_with_sink;
