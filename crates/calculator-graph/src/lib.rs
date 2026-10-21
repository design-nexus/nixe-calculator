pub mod calculus;
pub mod function;
pub mod sampler;
pub mod viewport;

pub use calculus::{derivative, find_extrema, find_intersections, find_roots, integrate};
pub use function::{DEFAULT_COLORS, PlottedFunction};
pub use sampler::{Polyline, sample_derivative, sample_function, sample_polar};
pub use viewport::Viewport;
