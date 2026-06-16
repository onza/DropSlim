mod collect;
mod events;
mod image;
mod output_path;
mod processor;
mod summary;
mod tools;
pub mod formats;

pub use events::{EventSink, ProcessorEvent, RecordingEventSink};
pub use image::optimize_image_file;
pub use output_path::UserSettings;
pub use processor::{process_paths, process_paths_with_sink};
