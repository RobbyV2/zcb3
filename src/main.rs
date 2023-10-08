// hide cmd window on windows
// #![windows_subsystem = "windows"]

mod bot;
mod gui;
pub use bot::*;

use clap::Parser;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub mod built_info {
    // the file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Run without any arguments to launch GUI.", long_about = None)]
struct Args {
    #[arg(long, short = 'r', help = "Path to replay file")]
    replay: String,
    #[arg(long, short = 'c', help = "Path to clickpack folder")]
    clicks: String,
    #[arg(
        long,
        short = 't',
        help = "Soft threshold for clicks (time between previous action and soft click in seconds)",
        default_value_t = 0.15
    )]
    soft_threshold: f32,
    #[arg(
        long,
        short = 'v',
        help = "Maximum random volume variation (+/-) of each click",
        default_value_t = 0.0
    )]
    volume_variation: f32,
    #[arg(
        long,
        short = 'n',
        help = "Whether to overlay the noise.* file in the clickpack directory",
        default_value_t = false
    )]
    noise: bool,
    #[arg(long, short, help = "Path to output file", default_value_t = String::from("output.wav"))]
    output: String,
    #[arg(
        long,
        short = 'm',
        help = "Whether to normalize the output audio (make all samples to be in range of 0-1)",
        default_value_t = false
    )]
    normalize: bool,
}

fn main() {
    env_logger::init(); // set envvar RUST_LOG=debug to see logs

    if std::env::args().len() > 1 {
        // we have arguments, probably need to run in cli mode
        let args = Args::parse();
        log::info!("passed args: {args:?} (running in cli mode)");
        run_cli(args);
    } else {
        log::info!("no args, running gui. pass -h or --help to see help");
        gui::run_gui().unwrap();
    }
}

/// Run command line interface
fn run_cli(mut args: Args) {
    // read replay
    let mut f = std::fs::File::open(args.replay.clone()).expect("failed to open replay file");
    let mut replay = String::new();
    f.read_to_string(&mut replay)
        .expect("failed to read replay file");

    let replay_filename = Path::new(&args.replay)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    // create bot (loads clickpack)
    let mut bot = Bot::new(PathBuf::from(args.clicks));

    // parse replay
    let replay = Macro::parse(
        MacroType::guess_format(&replay, replay_filename).unwrap(),
        &replay,
        args.soft_threshold,
    )
    .unwrap();

    // render output file
    let mut segment = bot.render_macro(replay, args.noise, args.volume_variation);

    if args.normalize {
        segment.normalize();
    }

    // save
    if args.output.is_empty() {
        log::warn!("output path is empty, defaulting to 'output.wav'");
        args.output = String::from("output.wav"); // can't save to empty path
    } else if !args.output.ends_with(".wav") {
        log::warn!("output path is not a .wav, however the output format is always .wav");
    }

    let f = std::fs::File::create(args.output).unwrap();
    segment.export_wav(f).unwrap();
}
