use crate::function::PlottedFunction;
use crate::viewport::Viewport;

pub type Polyline = Vec<(f64, f64)>;

pub fn sample_function(func: &PlottedFunction, vp: &Viewport, width: f64, height: f64) -> Vec<Polyline> {
    if !func.visible || func.parsed.is_none() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut current_segment: Polyline = Vec::new();

    // Oversample slightly (e.g. 2 samples per pixel) for ultra smooth curves
    let num_samples = (width * 2.0).max(100.0) as usize;
    let dx = vp.width_world() / num_samples as f64;

    let mut prev_y: Option<f64> = None;
    let mut prev_screen_y: Option<f64> = None;

    for i in 0..=num_samples {
        let x = vp.x_min + i as f64 * dx;
        match func.eval(x) {
            Ok(y) if y.is_finite() => {
                let (sx, sy) = vp.world_to_screen(x, y, width, height);

                // Discontinuity check (e.g. tan(x), 1/x asymptotes)
                let mut is_discontinuous = false;
                if let (Some(py), Some(psy)) = (prev_y, prev_screen_y) {
                    let y_sign_flip = (y * py) < 0.0;
                    let big_jump = (sy - psy).abs() > (height * 0.75);
                    if y_sign_flip && big_jump {
                        is_discontinuous = true;
                    }
                }

                if is_discontinuous && !current_segment.is_empty() {
                    segments.push(current_segment);
                    current_segment = Vec::new();
                }

                // Clamp extreme screen coordinates to avoid Cairo coordinate overflow
                let clamped_sy = sy.clamp(-height * 2.0, height * 3.0);
                current_segment.push((sx, clamped_sy));
                prev_y = Some(y);
                prev_screen_y = Some(sy);
            }
            _ => {
                // Undefined domain or evaluation error: break segment
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                    current_segment = Vec::new();
                }
                prev_y = None;
                prev_screen_y = None;
            }
        }
    }

    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    segments
}

pub fn sample_derivative(func: &PlottedFunction, vp: &Viewport, width: f64, height: f64) -> Vec<Polyline> {
    if !func.visible || !func.show_derivative || func.parsed.is_none() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut current_segment: Polyline = Vec::new();

    let num_samples = (width * 2.0).max(100.0) as usize;
    let dx = vp.width_world() / num_samples as f64;

    for i in 0..=num_samples {
        let x = vp.x_min + i as f64 * dx;
        match func.eval_derivative(x) {
            Ok(y) if y.is_finite() => {
                let (sx, sy) = vp.world_to_screen(x, y, width, height);
                let clamped_sy = sy.clamp(-height * 2.0, height * 3.0);
                current_segment.push((sx, clamped_sy));
            }
            _ => {
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                    current_segment = Vec::new();
                }
            }
        }
    }

    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    segments
}

pub fn sample_polar(
    func: &PlottedFunction,
    vp: &Viewport,
    width: f64,
    height: f64,
    theta_min: f64,
    theta_max: f64,
    steps: usize,
) -> Vec<Polyline> {
    if !func.visible || func.parsed.is_none() {
        return Vec::new();
    }

    let mut current_segment: Polyline = Vec::new();
    let dtheta = (theta_max - theta_min) / steps as f64;

    for i in 0..=steps {
        let theta = theta_min + i as f64 * dtheta;
        if let Ok(r) = func.eval(theta)
            && r.is_finite()
        {
            let x = r * theta.cos();
            let y = r * theta.sin();
            let (sx, sy) = vp.world_to_screen(x, y, width, height);
            current_segment.push((sx, sy));
            continue;
        }
    }

    if current_segment.is_empty() { Vec::new() } else { vec![current_segment] }
}
