mod animation;
mod cli;
mod mouse;

use mouse::{MouseExt, PointExt};

use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use fern::colors::{Color, ColoredLevelConfig};
use log::{error, warn, LevelFilter};

use std::time::Duration;

fn main() {
    let matches = cli::build().get_matches();
    init_logging(&matches);

    let interval = matches
        .get_one::<Duration>("INTERVAL")
        .expect("interval should be required by clap");
    let pause_interval = matches
        .get_one::<Duration>("pause-interval")
        .unwrap_or(interval);
    let fps = matches
        .get_one::<u32>("fps")
        .copied()
        .expect("fps should be required by clap");

    if !matches.get_flag("no-warn") {
        if matches.get_flag("no-autopause") && !matches.get_flag("no-animate") {
            warn!("auto-pause disabled with animations enabled, mouse is locked until the application exits");
        }
        if *interval > Duration::from_secs(60) {
            warn!(
                "interval ({}s) is longer than 1 minute, this may not be intentional",
                interval.as_secs()
            )
        }
        if fps > 500 {
            warn!("fps option may generate high CPU usage, if this becomes an issue consider lowering the value")
        }
    }

    let mouse = MouseExt::new(interval, pause_interval)
        .with_fps(fps)
        .with_animate(!matches.get_flag("no-animate"))
        .with_auto_pause(!matches.get_flag("no-autopause"));

    match run(mouse) {
        Ok(_) => (),
        Err(e) => error!("{e}"),
    }
}

fn init_logging(matches: &ArgMatches) {
    let level = if matches.get_flag("quiet") {
        LevelFilter::Off
    } else if matches.get_flag("verbose") {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let colors = ColoredLevelConfig::new()
        .trace(Color::Black)
        .debug(Color::Blue);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color}[{date}] {message}\x1B[0m",
                color = format_args!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str()),
                date = chrono::Local::now().format("%H:%M:%S"),
                message = message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}

fn run(mouse: MouseExt) -> Result<()> {
    let rng = fastrand::Rng::new();
    let mut orig = mouse
        .pos()
        .map_err(|_| anyhow!("failed to get mouse position"))?;

    loop {
        // TODO: use cli options for generating random points
        let p = PointExt {
            x: rng.i32((orig.x - 250)..(orig.x + 250)),
            y: rng.i32((orig.y - 250)..(orig.y + 250)),
        };

        match mouse.move_to(p) {
            Ok(_) => (),
            Err(err) => match err {
                mouse::MouseError::Busy => {
                    mouse.pause();
                    // use the new position as bounds since it was moved
                    orig = mouse
                        .pos()
                        .map_err(|_| anyhow!("failed to get mouse position"))?;
                }
                e => bail!("failed to move mouse ({e})"),
            },
        }
    }
}
