#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing { Linear, EaseOut, EaseInOut, Spring }

#[derive(Clone, Copy, Debug)]
pub struct Tween {
    pub from:    f32,
    pub to:      f32,
    pub current: f32,
    pub t:       f32,
    pub duration: f32,
    pub easing:  Easing,
}

impl Tween {
    pub fn new(from: f32, to: f32, duration_secs: f32, easing: Easing) -> Self {
        Self { from, to, current: from, t: 0.0, duration: duration_secs, easing }
    }

    pub fn instant(val: f32) -> Self {
        Self { from: val, to: val, current: val, t: 1.0, duration: 1.0, easing: Easing::Linear }
    }

    pub fn retarget(&mut self, to: f32) {
        self.from    = self.current;
        self.to      = to;
        self.t       = 0.0;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.t >= 1.0 { self.current = self.to; return; }
        self.t = (self.t + dt / self.duration).min(1.0);
        let p = apply(self.t, self.easing);
        self.current = self.from + (self.to - self.from) * p;
    }

    pub fn done(&self) -> bool { self.t >= 1.0 }
}

fn apply(t: f32, e: Easing) -> f32 {
    match e {
        Easing::Linear    => t,
        Easing::EaseOut   => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 },
        Easing::Spring    => {
            let s = t - 1.0;
            1.0 + s * s * ((2.0 * core::f32::consts::PI * 1.5).sin() * 0.3 + 1.0)
        }
    }
}
