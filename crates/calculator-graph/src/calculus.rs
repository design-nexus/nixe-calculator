use calculator_core::error::CalcError;

pub fn derivative<F: Fn(f64) -> Result<f64, CalcError>>(f: &F, x: f64) -> Result<f64, CalcError> {
    let h = 1e-6;
    let y1 = f(x + h)?;
    let y0 = f(x - h)?;
    Ok((y1 - y0) / (2.0 * h))
}

pub fn integrate<F: Fn(f64) -> Result<f64, CalcError>>(f: &F, a: f64, b: f64, n: usize) -> Result<f64, CalcError> {
    let n = if !n.is_multiple_of(2) { n + 1 } else { n };
    let h = (b - a) / n as f64;
    let mut sum = f(a)? + f(b)?;

    for i in 1..n {
        let x = a + i as f64 * h;
        let y = f(x)?;
        if i % 2 == 1 {
            sum += 4.0 * y;
        } else {
            sum += 2.0 * y;
        }
    }

    Ok(sum * (h / 3.0))
}

pub fn find_roots<F: Fn(f64) -> Result<f64, CalcError>>(f: &F, a: f64, b: f64, steps: usize) -> Vec<f64> {
    let mut roots = Vec::new();
    let dx = (b - a) / steps as f64;

    let mut prev_x = a;
    let mut prev_y = match f(a) {
        Ok(y) if y.is_finite() => y,
        _ => 0.0,
    };

    for i in 1..=steps {
        let x = a + i as f64 * dx;
        if let Ok(y) = f(x)
            && y.is_finite()
        {
            if y.abs() < 1e-9 {
                roots.push(x);
            } else if (prev_y * y) < 0.0 && (y - prev_y).abs() < 1000.0 {
                // Sign change without massive asymptote jump: bisection refinement
                if let Some(root) = bisect_root(f, prev_x, x, 40) {
                    // Deduplicate close roots
                    if roots.last().is_none_or(|&r| (r - root).abs() > 1e-4) {
                        roots.push(root);
                    }
                }
            }
            prev_x = x;
            prev_y = y;
        }
    }

    roots
}

fn bisect_root<F: Fn(f64) -> Result<f64, CalcError>>(f: &F, mut left: f64, mut right: f64, max_iter: usize) -> Option<f64> {
    let mut f_left = f(left).ok()?;
    for _ in 0..max_iter {
        let mid = (left + right) / 2.0;
        let f_mid = f(mid).ok()?;

        if f_mid.abs() < 1e-9 || (right - left).abs() < 1e-9 {
            return Some(mid);
        }

        if (f_left * f_mid) < 0.0 {
            right = mid;
        } else {
            left = mid;
            f_left = f_mid;
        }
    }
    Some((left + right) / 2.0)
}

pub fn find_extrema<F: Fn(f64) -> Result<f64, CalcError>>(f: &F, a: f64, b: f64, steps: usize) -> Vec<(f64, f64, bool)> {
    // Extrema are roots of the derivative f'(x) = 0
    let df = |x: f64| derivative(f, x);
    let candidate_xs = find_roots(&df, a, b, steps);

    let mut extrema = Vec::new();
    for x in candidate_xs {
        if let Ok(y) = f(x) {
            let h = 1e-4;
            if let (Ok(y_left), Ok(y_right)) = (f(x - h), f(x + h)) {
                let is_max = y > y_left && y > y_right;
                let is_min = y < y_left && y < y_right;
                if is_max || is_min {
                    extrema.push((x, y, is_max));
                }
            }
        }
    }
    extrema
}

pub fn find_intersections<F1, F2>(f1: &F1, f2: &F2, a: f64, b: f64, steps: usize) -> Vec<(f64, f64)>
where
    F1: Fn(f64) -> Result<f64, CalcError>,
    F2: Fn(f64) -> Result<f64, CalcError>,
{
    let diff = |x: f64| {
        let y1 = f1(x)?;
        let y2 = f2(x)?;
        Ok(y1 - y2)
    };

    let roots = find_roots(&diff, a, b, steps);
    let mut intersections = Vec::new();
    for x in roots {
        if let Ok(y) = f1(x) {
            intersections.push((x, y));
        }
    }
    intersections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivative() {
        // d/dx(x^2) at x=3 is 6
        let f = |x: f64| Ok(x * x);
        let df = derivative(&f, 3.0).unwrap();
        assert!((df - 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_integration() {
        // int_0^2 x dx = 2.0
        let f = |x: f64| Ok(x);
        let area = integrate(&f, 0.0, 2.0, 100).unwrap();
        assert!((area - 2.0).abs() < 1e-4);

        // int_0^1 x^2 dx = 1/3 ≈ 0.333333
        let f_sq = |x: f64| Ok(x * x);
        let area_sq = integrate(&f_sq, 0.0, 1.0, 100).unwrap();
        assert!((area_sq - 1.0 / 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_root_finding() {
        // x^2 - 4 has roots at -2 and 2
        let f = |x: f64| Ok(x * x - 4.0);
        let roots = find_roots(&f, -5.0, 5.0, 200);
        assert_eq!(roots.len(), 2);
        assert!((roots[0] - (-2.0)).abs() < 1e-3);
        assert!((roots[1] - 2.0).abs() < 1e-3);
    }

    #[test]
    fn test_extrema() {
        // -x^2 + 4 has maximum at (0, 4)
        let f = |x: f64| Ok(-x * x + 4.0);
        let extrema = find_extrema(&f, -5.0, 5.0, 200);
        assert_eq!(extrema.len(), 1);
        assert!(extrema[0].0.abs() < 1e-3);
        assert!((extrema[0].1 - 4.0).abs() < 1e-3);
        assert!(extrema[0].2); // is_max
    }
}
