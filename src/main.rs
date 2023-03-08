mod animation;
mod bounds;
mod cli;
mod mouse;

use bounds::Bounds;
use mouse::{MouseExt, PointExt};

use anyhow::{anyhow, bail, Result};
use crossterm::cursor::MoveToColumn;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor, Stylize};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use std::io::stdout;

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

    let mouse = MouseExt::new(interval, pause_interval)
        .with_fps(fps)
        .with_animate(!matches.get_flag("no-animate"))
        .with_auto_pause(!matches.get_flag("no-autopause"));

    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Hide,
        EnterAlternateScreen,
        Clear(ClearType::All),
    )
    .expect("should be able to execute crossterm commands");
    enable_raw_mode().expect("should be able to start raw mode");

    let code = match run(&mouse, &bounds) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    };

    disable_raw_mode().expect("should be able to disable raw mode");
    execute!(stdout, cursor::Show, LeaveAlternateScreen)
        .expect("should be able to leave alternate screen");

    code
}

fn run(mouse: &MouseExt, bounds: &Bounds) -> Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        Print("Application started.\n".dim()),
        cursor::MoveToColumn(0),
        Print("Press ".dim()),
        Print("q".bold()),
        Print(" to quit\n\n".dim()),
        cursor::MoveToColumn(0),
    )?;

    let rng = fastrand::Rng::new();
    let mut orig = mouse
        .pos()
        .map_err(|_| anyhow!("failed to get mouse position"))?;

    let poll_time = if mouse.animated() {
        Duration::from_millis(25)
    } else {
        mouse.interval().to_owned()
    };

    let mut last_p = orig;
    loop {
        if poll(poll_time)? {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => return Ok(()),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => return Ok(()),
                _ => (),
            };
        }

        let p = sample_point(&rng, bounds, orig, last_p);
        execute!(
            stdout,
            Clear(ClearType::CurrentLine),
            Print("Status:".bold()),
            SetForegroundColor(Color::Cyan),
            Print(" moving cursor to "),
            Print(p),
            ResetColor,
            MoveToColumn(0),
        )?;

        match mouse.move_to(p) {
            Ok(_) => (),
            Err(err) => match err {
                mouse::MouseError::Busy => {
                    let pause_str = format!("{:.1}s", mouse.pause_interval().as_secs_f32());
                    execute!(
                        stdout,
                        Clear(ClearType::CurrentLine),
                        Print("Status:".bold()),
                        SetForegroundColor(Color::Yellow),
                        Print(" mouse busy, pausing for "),
                        Print(pause_str),
                        ResetColor,
                        MoveToColumn(0),
                    )?;
                    mouse.auto_pause();
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

fn sample_point(
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
