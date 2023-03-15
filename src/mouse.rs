use crate::{animation, config::Config, input};

use mouse_rs::types::Point;
use mouse_rs::Mouse;
use thiserror::Error;

use std::{
    fmt,
    time::{Duration, Instant},
};

const AUTO_PAUSE_TOLERANCE: f64 = 50.0;

#[derive(Debug, Error)]
pub enum MouseError {
    #[error("mouse was in use")]
    Busy,
    #[error("internal error: {0}")]
    InternalError(#[from] Box<dyn std::error::Error>),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PointExt {
    pub x: i32,
    pub y: i32,
}

impl fmt::Display for PointExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
    }
}

impl PointExt {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn is_near(&self, p: Self, tolerance: f64) -> bool {
        f64::sqrt(f64::powi((self.x - p.x) as f64, 2) + f64::powi((self.y - p.y) as f64, 2))
            < tolerance
    }

    pub fn lerp(p1: Self, p2: Self, t: f64) -> Self {
        let t_clamp = t.clamp(0., 1.);

        Self::new(
            (p1.x as f64 + (p2.x - p1.x) as f64 * t_clamp).round() as i32,
            (p1.y as f64 + (p2.y - p1.y) as f64 * t_clamp).round() as i32,
        )
    }
}

impl From<Point> for PointExt {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

pub struct MouseExt {
    inner: Mouse,
    interval: Duration,
    pause_interval: Duration,
    fps: u32,
    animate: bool,
    auto_pause: bool,
}

impl MouseExt {
    pub fn with_config(config: &Config) -> Self {
        Self {
            inner: Mouse::new(),
            interval: config.interval,
            pause_interval: config.pause_interval,
            fps: config.fps,
            animate: config.animate,
            auto_pause: config.auto_pause,
        }
    }

    #[inline]
    pub fn pos(&self) -> Result<PointExt, MouseError> {
        Ok(self.inner.get_position()?.into())
    }

    pub fn toggle_animate(&mut self) {
        self.animate = !self.animate;
    }

    pub fn move_to(&self, p: PointExt) -> Result<(), MouseError> {
        if !self.animate {
            return self.move_to_no_animate(p);
        }

        let frame_ms = 1000. / self.fps as f64;
        let frame_time = Duration::from_millis(frame_ms.round() as u64);

        let start_pos = self.pos()?;
        let mut last_pos = start_pos;
        let mut elapsed = Duration::from_secs(0);

        while elapsed < self.interval {
            let f_start = Instant::now();

            // note: macOS `get_position` implementation seems to not update
            // fast enough for animating. using the `is_near` method allows some
            // level of tolerance for the animation to continue, but will still
            // correctly stop if the user moves the mouse around to unlock it
            let curr_pos = self.pos()?;
            if self.auto_pause && !last_pos.is_near(curr_pos, AUTO_PAUSE_TOLERANCE) {
                return Err(MouseError::Busy);
            }

            // interpolate the animation
            let t = elapsed.as_millis() as f64 / self.interval.as_millis() as f64;
            let new_pos = PointExt::lerp(start_pos, p, animation::ease_in_out(t));

            // only update mouse if the position will change
            if new_pos != last_pos {
                self.inner.move_to(new_pos.x, new_pos.y)?;
                last_pos = self.pos()?;
            }

            // pause for the remainder of frame time to achieve target fps
            let dt = f_start.elapsed();
            if dt < frame_time {
                spin_sleep::sleep(frame_time - dt);
                // make sure stdin isn't waiting while animating
                if input::is_stdin_waiting(Duration::from_secs(0)) {
                    return Ok(());
                }
            }

            elapsed += f_start.elapsed();
        }

        Ok(())
    }

    fn move_to_no_animate(&self, p: PointExt) -> Result<(), MouseError> {
        self.inner.move_to(p.x, p.y)?;

        // make sure stdin isn't waiting while pausing
        if input::is_stdin_waiting(self.interval) {
            return Ok(());
        }

        if self.auto_pause && !self.pos()?.is_near(p, AUTO_PAUSE_TOLERANCE) {
            return Err(MouseError::Busy);
        }

        Ok(())
    }

    pub fn auto_pause(&self) {
        // TODO: this should poll the mouse location on a short interval to reset the
        //   timer if the mouse is in use while auto-pausing
        if self.auto_pause && input::is_stdin_waiting(self.pause_interval) {
            // block intentionally empty
        }
    }
}
