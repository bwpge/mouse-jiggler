/// Linearly interpolates a value between `[min, max]`, given a `t` between
/// `[0, 1]`.
///
/// Note that `t` is clamped between `0` and `1` (inclusive).
#[inline]
pub fn lerp(min: f64, max: f64, t: f64) -> f64 {
    let t_clamp = t.clamp(0., 1.);

    min + (max - min) * t_clamp
}

/// Eases `t` using simple quadratic functions. The result is a soft
/// acceleration and deceleration when `t` is near `0` or `1` respectively.
#[inline]
pub fn ease_in_out(t: f64) -> f64 {
    let in_t = ease_in(t);
    let out_t = ease_out(t);

    lerp(in_t, out_t, t)
}

#[inline]
fn ease_in(t: f64) -> f64 {
    square(t)
}

#[inline]
fn ease_out(t: f64) -> f64 {
    flip(square(flip(t)))
}

#[inline]
fn flip(t: f64) -> f64 {
    1. - t
}

#[inline]
fn square(t: f64) -> f64 {
    t * t
}
