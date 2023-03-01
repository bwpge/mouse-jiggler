mod animation;
mod bounds;
mod cli;
mod mouse;

use bounds::Bounds;
use mouse::{MouseExt, PointExt};

use anyhow::{anyhow, bail, Result};

use std::process::ExitCode;
use std::time::Duration;

fn main() -> ExitCode {
    let matches = cli::build().get_matches();

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
    let bounds = Bounds::from(&matches);
    if bounds.has_empty_range() {
        eprintln!("error: bounds {bounds} will result in no mouse movement");
        return ExitCode::FAILURE;
    }

    if matches.get_flag("no-autopause") && !matches.get_flag("no-animate") {
        eprintln!("warning: auto-pause disabled with animations enabled, mouse is locked until the application exits");
    }

    let mouse = MouseExt::new(interval, pause_interval)
        .with_fps(fps)
        .with_animate(!matches.get_flag("no-animate"))
        .with_auto_pause(!matches.get_flag("no-autopause"));

    match run(&mouse, &bounds) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(mouse: &MouseExt, bounds: &Bounds) -> Result<()> {
    let rng = fastrand::Rng::new();
    let mut orig = mouse
        .pos()
        .map_err(|_| anyhow!("failed to get mouse position"))?;

    let mut last_p = orig;
    loop {
        let p = gen_new_point(&rng, &bounds, orig, last_p);

        match mouse.move_to(p) {
            Ok(_) => (),
            Err(err) => match err {
                mouse::MouseError::Busy => {
                    mouse.pause();
                    // use the new position as bounds since it was moved
                    if bounds.is_relative() {
                        orig = mouse
                            .pos()
                            .map_err(|_| anyhow!("failed to get mouse position"))?;
                    }
                }
                e => bail!("failed to move mouse ({e})"),
            },
        }

        last_p = p;
    }
}

fn gen_new_point(
    rng: &fastrand::Rng,
    bounds: &Bounds,
    orig: PointExt,
    last_p: PointExt,
) -> PointExt {
    loop {
        let result = match *bounds {
            Bounds::Rect { x1, y1, x2, y2 } => {
                let x_range = if x1 <= x2 { x1..=x2 } else { x2..=x1 };
                let y_range = if y1 <= y2 { y1..=y2 } else { y2..=y1 };
                PointExt {
                    x: rng.i32(x_range),
                    y: rng.i32(y_range),
                }
            }
            Bounds::Relative { dx: x, dy: y } => PointExt {
                x: rng.i32((orig.x - x)..=(orig.x + x)),
                y: rng.i32((orig.y - y)..=(orig.y + y)),
            },
        };

        if result != last_p {
            return result;
        }
    }
}
