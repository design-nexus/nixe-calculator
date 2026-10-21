use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { x_min: -10.0, x_max: 10.0, y_min: -10.0, y_max: 10.0 }
    }
}

impl Viewport {
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self { x_min, x_max, y_min, y_max }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn width_world(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height_world(&self) -> f64 {
        self.y_max - self.y_min
    }

    pub fn world_to_screen(&self, x: f64, y: f64, width: f64, height: f64) -> (f64, f64) {
        let sx = (x - self.x_min) / self.width_world() * width;
        let sy = (self.y_max - y) / self.height_world() * height;
        (sx, sy)
    }

    pub fn screen_to_world(&self, sx: f64, sy: f64, width: f64, height: f64) -> (f64, f64) {
        let wx = self.x_min + (sx / width) * self.width_world();
        let wy = self.y_max - (sy / height) * self.height_world();
        (wx, wy)
    }

    pub fn pan(&mut self, dx_px: f64, dy_px: f64, width: f64, height: f64) {
        let dx_w = dx_px * self.width_world() / width;
        let dy_w = dy_px * self.height_world() / height;
        self.x_min -= dx_w;
        self.x_max -= dx_w;
        self.y_min += dy_w;
        self.y_max += dy_w;
    }

    pub fn zoom(&mut self, factor: f64, center_px: Option<(f64, f64)>, width: f64, height: f64) {
        let (anchor_x, anchor_y) = if let Some((cx, cy)) = center_px {
            self.screen_to_world(cx, cy, width, height)
        } else {
            ((self.x_min + self.x_max) / 2.0, (self.y_min + self.y_max) / 2.0)
        };

        let new_w = self.width_world() * factor;
        let new_h = self.height_world() * factor;

        // Prevent zooming too far in or out
        if !(1e-9..=1e9).contains(&new_w) || !(1e-9..=1e9).contains(&new_h) {
            return;
        }

        let frac_x = (anchor_x - self.x_min) / self.width_world();
        let frac_y = (anchor_y - self.y_min) / self.height_world();

        self.x_min = anchor_x - frac_x * new_w;
        self.x_max = self.x_min + new_w;
        self.y_min = anchor_y - frac_y * new_h;
        self.y_max = self.y_min + new_h;
    }

    pub fn grid_step(&self, target_pixels: f64, screen_size: f64, world_size: f64) -> f64 {
        let rough_step = world_size / (screen_size / target_pixels);
        let magnitude = 10f64.powf(rough_step.log10().floor());
        let normalized = rough_step / magnitude;

        let nice_step = if normalized < 1.5 {
            1.0
        } else if normalized < 3.5 {
            2.0
        } else if normalized < 7.5 {
            5.0
        } else {
            10.0
        };

        nice_step * magnitude
    }

    pub fn grid_steps(&self, width: f64, height: f64) -> (f64, f64) {
        let step_x = self.grid_step(80.0, width, self.width_world());
        let step_y = self.grid_step(80.0, height, self.height_world());
        (step_x, step_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_transform_roundtrip() {
        let vp = Viewport::default();
        let (w, h) = (800.0, 600.0);

        let (sx, sy) = vp.world_to_screen(0.0, 0.0, w, h);
        assert!((sx - 400.0).abs() < 1e-6);
        assert!((sy - 300.0).abs() < 1e-6);

        let (wx, wy) = vp.screen_to_world(sx, sy, w, h);
        assert!(wx.abs() < 1e-6);
        assert!(wy.abs() < 1e-6);
    }

    #[test]
    fn test_viewport_zoom_and_pan() {
        let mut vp = Viewport::default();
        let (w, h) = (800.0, 600.0);

        vp.zoom(0.5, None, w, h);
        assert!((vp.width_world() - 10.0).abs() < 1e-6);
        assert!((vp.height_world() - 10.0).abs() < 1e-6);

        vp.pan(400.0, 0.0, w, h);
        assert!((vp.x_min - (-10.0)).abs() < 1e-6);
    }
}
