
mod buffer;
mod source;
mod error;
mod tui;
mod parser;
mod ansi;
mod string;

use crate::source::{AsyncPipeIn, AsyncFileIn, Source};
use crate::error::Error;
use crate::tui::{Tui, Mode};

use clap::{Arg, App, ArgMatches};

const TITLE: &str = "Log View";
const VERSION: &str = "0.1";
const AUTHOR: &str = "David Loewe <49597367+TumbleOwlee@users.noreply.github.com>";
const FILE: &str = "file";
const THEME: &str = "theme";
const RETAIN_COLORS: &str = "retain-colors";
const SKIP_COLOR_CHECK: &str = "skip-color-check";
const BUFFER_SIZE: usize = 1024;
const HISTORY: &str = "history";

fn main() -> Result<(), Error>{
    App::new(TITLE)
        .version(VERSION)
        .author(AUTHOR)
        .about("Filter active logs")
        .arg(Arg::with_name(FILE)
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Sets the input file")
            .takes_value(true))
        .arg(Arg::with_name(THEME)
            .short("t")
            .long("theme")
            .value_name("THEME")
            .help("Set custom theme")
            .takes_value(true))
        .arg(Arg::with_name(RETAIN_COLORS)
            .short("c")
            .long("color")
            .takes_value(false)
            .help("Retain input colors"))
        .arg(Arg::with_name(SKIP_COLOR_CHECK)
            .short("s")
            .long("skip")
            .takes_value(false)
            .help("Skip color check"))
        .arg(Arg::with_name(HISTORY)
            .long("history")
            .takes_value(true)
            .value_name("DESTINATION")
            .help("Store history on quit"))
        .get_matches_safe()
        .map_err(|e| e.exit())
        .map(start)
        .unwrap()
}

fn start(matches: ArgMatches) -> Result<(), Error> {
    matches.value_of(FILE)
        .map_or_else(|| AsyncPipeIn::start().map(Source::from),
            |f| AsyncFileIn::start(f).map(Source::from))
        .map(|src| {
            let mut tui = Tui::new()
                .set_color_mode(
                    if matches.is_present(RETAIN_COLORS) {
                        Mode::RetainColors
                    } else if matches.is_present(SKIP_COLOR_CHECK) {
                        Mode::SkipColorCheck
                    } else {
                        Mode::RemoveColors
                    }
                );
            if let Some(p) = matches.value_of(HISTORY) {
                tui.set_history_path(p.into());
            }
            if let Some(th) = matches.value_of(THEME) {
                tui.use_custom_theme(th)?;
            } else {
                tui.use_default_theme();
            }
            tui.run(src);
            Ok(())
        })
        .unwrap_or_else(Err)
}

