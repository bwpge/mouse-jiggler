use crate::animation;

use log::{info, trace};
use mouse_rs::types::Point;
use mouse_rs::Mouse;
use thiserror::Error;

use std::time::{Duration, Instant};

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

impl PointExt {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn lerp(p1: PointExt, p2: PointExt, t: f64) -> PointExt {
        let t_clamp = t.clamp(0., 1.);

        PointExt::new(
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
    pub fn new(interval: &Duration, pause_interval: &Duration) -> Self {
        Self {
            inner: Mouse::new(),
            interval: interval.to_owned(),
            pause_interval: pause_interval.to_owned(),
            fps: 144,
            animate: true,
            auto_pause: true,
        }
    }

    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    pub fn with_animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    pub fn with_auto_pause(mut self, auto_pause: bool) -> Self {
        self.auto_pause = auto_pause;
        self
    }

    pub fn pos(&self) -> Result<PointExt, MouseError> {
        Ok(self.inner.get_position()?.into())
    }

    pub fn move_to(&self, p: PointExt) -> Result<(), MouseError> {
        if !self.animate {
            return self.move_to_no_animate(p);
        }

        info!("Moving mouse to {}, {}", p.x, p.y);

        let frame_ms = 1000. / self.fps as f64;
        let frame_time = Duration::from_millis(frame_ms.round() as u64);

        let start_pos = self.pos()?;
        let mut last_pos = start_pos;
        let mut elapsed = Duration::from_secs(0);
        let mut frame = 0;

        while elapsed < self.interval {
            frame += 1;
            let f_start = Instant::now();

            let curr_pos = self.pos()?;
            if self.auto_pause && last_pos != curr_pos {
                return Err(MouseError::Busy);
            }

            // interpolate the animation
            let t = elapsed.as_millis() as f64 / self.interval.as_millis() as f64;
            let new_pos = PointExt::lerp(start_pos, p, animation::ease_in_out(t));

            // only update mouse if the position will change
            if new_pos != last_pos {
                trace!(
                    "Animating movement (x={}, y={}, t={t:0.4}, frame={frame})",
                    new_pos.x,
                    new_pos.y
                );
                self.inner.move_to(new_pos.x, new_pos.y)?;
                last_pos = self.pos()?;
            }

            // pause for the remainder of frame time to achieve target fps
            let dt = Instant::now() - f_start;
            if dt < frame_time {
                spin_sleep::sleep(frame_time - dt);
            }

            elapsed += Instant::now() - f_start;
        }

        Ok(())
    }

    fn move_to_no_animate(&self, p: PointExt) -> Result<(), MouseError> {
        let start_pos = self.pos()?;
        spin_sleep::sleep(self.interval);

        if self.auto_pause && self.pos()? != start_pos {
            return Err(MouseError::Busy);
        }

        info!("Moving mouse to {}, {}", p.x, p.y);
        self.inner.move_to(p.x, p.y)?;

        Ok(())
    }

    pub fn pause(&self) {
        if self.auto_pause {
            info!(
                "Mouse was in use, pausing movement for {}s",
                self.pause_interval.as_secs_f32()
            );
            spin_sleep::sleep(self.pause_interval);
        }
    }
}
