mod buffer;
mod error;
mod parser;
mod source;
mod string;
mod tui;

use crate::error::Error;
use crate::source::{AsyncFileIn, AsyncPipeIn, Source};
use crate::tui::{Mode, Tui};

use clap::Parser;

/// Simple program to view logs with regex
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: Option<String>,

    /// Custom theme
    #[arg(short, long)]
    theme: Option<String>,

    /// Retain input colors
    #[arg(short, long)]
    color: bool,

    /// Skip color check
    #[arg(short, long)]
    skip: bool,

    /// Store history on quit
    #[arg(long)]
    history: Option<String>,
}

fn main() -> Result<(), Error> {
    start(Args::parse())
}

fn start(args: Args) -> Result<(), Error> {
    let src = match &args.file {
            None => AsyncPipeIn::start().map(Source::from),
            Some(f) => AsyncFileIn::start(f).map(Source::from),
    }?;

    let retain = match (args.color, args.skip) {
        (true, _) => Mode::RetainColors,
        (false, true) => Mode::SkipColorCheck,
        (_, _) => Mode::RemoveColors,
    };

    let mut tui = Tui::new().set_color_mode(retain);

    if let Some(p) = &args.history {
        tui.set_history_path(p.into());
    }

    if let Some(t) = &args.theme {
        tui.use_custom_theme(t)?;
    } else {
        tui.use_default_theme();
    }

    tui.run(src);
    Ok(())

}
