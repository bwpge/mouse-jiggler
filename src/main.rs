mod animation;
mod cli;
mod mouse;

use mouse::{MouseExt, PointExt};

use anyhow::{anyhow, bail, Result};
use log::{info, warn, error};

use std::time::Duration;

fn main() {
    let matches = cli::build().get_matches();
    let interval = matches
        .get_one::<Duration>("INTERVAL")
        .expect("interval should be required by clap");
    let fps = matches
        .get_one::<u32>("fps")
        .copied()
        .expect("fps should be required by clap");

    if *interval > Duration::from_secs(3600) {
        warn!("input interval is longer than 1 hour (got: {} seconds)", interval.as_secs())
    }
    if fps > 200 {
        warn!("fps option may generate high CPU usage, if this becomes an issue consider lowering the value")
    }

    let mouse = MouseExt::new(interval)
        .with_fps(fps)
        .with_animate(!matches.get_flag("no-animate"))
        .with_auto_pause(!matches.get_flag("no-autopause"));

    match run(mouse) {
        Ok(_) => (),
        Err(e) => error!("{e}"),
    }
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
                    info!("Mouse was in use, pausing for one interval");
                    mouse.pause();
                    // use the new position as bounds since it was moved
                    // TODO: fix unwrapping here
                    orig = mouse.pos().unwrap();
                },
                e => bail!("failed to move mouse ({e})"),
            },
        }
    }
}
