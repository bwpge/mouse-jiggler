use anyhow::{anyhow, ensure, Result};
use clap::{arg, builder::ValueParser, command, ArgAction, Command, ArgGroup, Arg, value_parser};

use std::time::Duration;

const INTERVAL_LONG_HELP: &str = "Specify how much time should elapse between \
mouse movements. If not specified, defaults to 1 second.

A single number is parsed as SECONDS between movements. Numbers can \
be specified as integers (e.g., 42) or floating point numbers (e.g., 0.42). \
A single number argument must be a positive value.

Warnings are emitted for any duration longer than 1 hour, as this is probably \
not intentional for most use cases.";

const NO_ANIMATE_LONG_HELP: &str = "Do not animate mouse movements. Instead, \
'place' the mouse at each point.

Note: some applications may detect this as 'botting' or unusual input. If you 
are using this utility to prevent away statuses from triggering, this option \
is not recommended.";

const NO_AUTO_PAUSE_LONG_HELP: &str = "Do not pause mouse movements if the mouse is in use.

This options is helpful if you want to ensure the mouse is always moved in the \
calculated region. For example, if your physical mouse might be bumped or moved, \
you can use this option to ensure the mouse will continue moving where expected.

WARNING: If '-a' is NOT specified, you won't be able to move your mouse until \
this application is closed.";

pub fn build() -> Command {
    command!()
        .disable_help_flag(true)
        .disable_version_flag(true)
        .after_help("Use '--help' for detailed information")
        .after_long_help("Use '-h' for brief information")
        .arg(
            arg!([INTERVAL] "Duration between movements (see '--help' for formatting)")
                .long_help(INTERVAL_LONG_HELP)
                .default_value("1")
                .hide_default_value(true)
                .value_parser(ValueParser::new(parse_interval)),
        )
        .next_help_heading("Mouse Options")
        .arg(Arg::new("bounds")
            .short('b')
            .long("bounds")
            .help("Restrict mouse movements inside a rectangle")
            .num_args(4)
            .value_names(["X1", "Y1", "X2", "Y2"])
            .value_delimiter(',')
            .value_parser(value_parser!(i32))
            .conflicts_with("relative-bounds"))
        .arg(Arg::new("relative-bounds")
            .short('r')
            .long("relative-bounds")
            .help("Generate random points relative (+/- X and Y) to the current mouse position")
            .num_args(2)
            .value_delimiter(',')
            .value_parser(value_parser!(i32))
            .value_names(["X", "Y"]))
        .arg(
            arg!(-f --fps <FPS> "Number of animation frames per second (default: 60)")
                .default_value("60")
                .hide_default_value(true)
                .value_parser(ValueParser::new(parse_fps))
                .conflicts_with("no-animate"),
        )
        .arg(
            arg!(-a --"no-animate" "Do not animate mouse movements")
                .long_help(NO_ANIMATE_LONG_HELP),
        )
        .arg(
            arg!(-p --"no-autopause" "Do not pause mouse movements if the mouse is in use")
                .long_help(NO_AUTO_PAUSE_LONG_HELP),
        )
        .next_help_heading("Output Options")
        .arg(arg!(-q --quiet "Suppress all output"))
        .arg(arg!(-w --"no-warn" "Do not emit any warnings"))
        .arg(arg!(-v --verbose "Use verbose log output"))
        .group(ArgGroup::new("verbosity").args(["quiet", "no-warn", "verbose"]))
        .next_help_heading("Options")
        .arg(arg!(-h --help "Print help information and quit").action(ArgAction::Help))
        .arg(arg!(-V --version "Print version information and quit").action(ArgAction::Version))
}

pub fn parse_interval(s: &str) -> Result<Duration> {
    if let Ok(result) = parse_sec_u64(s) {
        return Ok(result);
    }

    if let Ok(result) = parse_sec_f64(s) {
        return Ok(result);
    }

    Err(anyhow!("could not parse input as an interval"))
}

fn parse_sec_u64(s: &str) -> Result<Duration> {
    match s.parse::<u64>() {
        Ok(value) => {
            ensure!(value > 0, "interval must be a positive number");
            Ok(Duration::from_secs(value))
        }
        Err(e) => Err(anyhow!(e)),
    }
}

fn parse_sec_f64(s: &str) -> Result<Duration> {
    match s.parse::<f64>() {
        Ok(value) => {
            ensure!(value > 0., "interval must be a positive number");
            let ms = value * 1000.;
            Ok(Duration::from_millis(ms.round() as u64))
        }
        Err(e) => Err(anyhow!(e)),
    }
}

fn parse_fps(s: &str) -> Result<u32> {
    // parse first as i64 so we can report better error messages
    match s.parse::<i64>() {
        Ok(value) => {
            ensure!(value > 0, "fps must be a positive number");
            ensure!(
                value <= u32::MAX as i64,
                format!("fps must be between 1 and {}", u32::MAX)
            );
            Ok(value as u32)
        }
        Err(e) => Err(anyhow!(e)),
    }
}
