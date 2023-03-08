use crate::bounds::Bounds;

use std::time::Duration;

pub struct Config {
    pub interval: Duration,
    pub pause_interval: Duration,
    pub fps: u32,
    pub bounds: Bounds,
    pub animate: bool,
    pub auto_pause: bool,
}
