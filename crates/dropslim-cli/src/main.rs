mod exit;
mod project_root;
mod settings;
mod sink;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::{Parser, Subcommand, ValueEnum};
use dropslim_core::{process_paths_with_sink, OutputFormatSetting, UserSettings};

use crate::exit::{exit_code, RunOutcome};
use crate::project_root::resolve_project_root;
use crate::settings::{settings_from_flags, validate_out_dir};
use crate::sink::CliEventSink;

#[derive(Debug, Parser)]
#[command(
    name = "dropslim",
    version,
    about = "Optimize images from the command line",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Compress and convert images
    Compress(CompressArgs),
}

#[derive(Debug, Parser)]
struct CompressArgs {
    /// Input files or folders
    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,

    /// Overwrite inputs instead of writing *.min.* beside them
    #[arg(long)]
    no_suffix: bool,

    /// Write into a minified/ subfolder
    #[arg(long)]
    subfolder: bool,

    /// Write all outputs into this folder
    #[arg(short = 'o', long = "out", value_name = "DIR")]
    out: Option<PathBuf>,

    /// Max output width (keeps aspect ratio)
    #[arg(long, value_name = "PX")]
    max_width: Option<u32>,

    /// Max output height (keeps aspect ratio)
    #[arg(long, value_name = "PX")]
    max_height: Option<u32>,

    /// Output format (default: keep original)
    #[arg(long, value_enum, default_value_t = FormatArg::Original)]
    format: FormatArg,

    /// Path to gifsicle (overrides DROPSLIM_GIFSICLE and vendor/PATH lookup)
    #[arg(long, value_name = "PATH")]
    gifsicle: Option<PathBuf>,

    /// Emit one JSON object per event on stdout (progress on stderr)
    #[arg(long)]
    json: bool,

    /// Only print errors (and JSON events that are errors/completion)
    #[arg(short = 'q', long)]
    quiet: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    Original,
    Jpeg,
    Png,
    Webp,
    Avif,
}

impl From<FormatArg> for OutputFormatSetting {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Original => Self::Original,
            FormatArg::Jpeg => Self::Jpeg,
            FormatArg::Png => Self::Png,
            FormatArg::Webp => Self::Webp,
            FormatArg::Avif => Self::Avif,
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compress(args) => ExitCode::from(run_compress(args).await),
    }
}

async fn run_compress(args: CompressArgs) -> u8 {
    if let Err(error) = validate_out_dir(args.out.as_deref()) {
        eprintln!("error: {error}");
        return 2;
    }

    if let Some(path) = args.gifsicle.as_ref() {
        // Process-local override before any worker starts.
        std::env::set_var("DROPSLIM_GIFSICLE", path);
    }

    let settings: UserSettings = settings_from_flags(
        args.no_suffix,
        args.subfolder,
        args.out.as_deref(),
        args.max_width,
        args.max_height,
        args.format.into(),
    );

    let cancel = Arc::new(AtomicBool::new(false));
    spawn_cancel_listener(Arc::clone(&cancel));

    let sink = Arc::new(CliEventSink::new(args.json, args.quiet));
    let input_paths: Vec<String> = args
        .paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    if let Err(error) = process_paths_with_sink(
        Arc::clone(&sink),
        input_paths,
        settings,
        resolve_project_root(),
        Arc::clone(&cancel),
    )
    .await
    {
        eprintln!("error: {error}");
        return 1;
    }

    let outcome = if sink.cancelled() || cancel.load(Ordering::SeqCst) {
        RunOutcome::Cancelled
    } else if sink.failed() > 0 {
        RunOutcome::Failed
    } else {
        RunOutcome::Ok
    };

    exit_code(outcome)
}

fn spawn_cancel_listener(cancel: Arc<AtomicBool>) {
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel.store(true, Ordering::SeqCst);
        }
    });
}
