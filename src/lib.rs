extern crate num_traits;
#[macro_use]
extern crate smallvec;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod polynomial;

use num_traits::{FromPrimitive, One, Zero};
use polynomial::Polynomial;
use std::fmt::Debug;
use std::ops::Div;

/// A 2d space curve
#[derive(Debug, Clone)]
pub struct Curve<T: Zero + One + Clone> {
    x: Polynomial<T>,
    y: Polynomial<T>,
}

impl<T: Zero + One + Clone> Curve<T> {
    /// Evaluates the curve at `t`. Properly defined only when `0 <= t <= 1`. Returns as `(x, y)`.
    // TODO: use Result
    pub fn eval(&self, t: T) -> (T, T) {
        (self.x.eval(t.clone()), self.y.eval(t))
    }
}

impl<T: Zero + One + Clone + PartialOrd + FromPrimitive + Div<Output = T>> Curve<T> {
    /// Returns a vector of points on the curve at equal parameter increments.
    pub fn render(&self, num_pts: usize) -> Vec<(T, T)> {
        let mut t: T = Zero::zero();
        let one: T = One::one();
        let step: T = one / T::from_usize(num_pts).unwrap();
        let mut pts = Vec::with_capacity(num_pts);
        while t < One::one() {
            pts.push(self.eval(t.clone()));
            t = t + step.clone();
        }
        pts
    }
}

/// Represents one interpolation state of an eta-3 spline.
/// Contains x position, y position, theta in radians, curvature, and the derivative of curvature.
pub struct MotionState<T: Zero + One + Clone> {
    pub x: T,
    pub y: T,
    pub t: T,
    pub k: T,
    pub dk: T,
}

#[derive(Debug, Clone)]
pub struct EtaParam<T: Zero>(T, T, T, T, T, T);

impl<T: Zero + Clone + PartialOrd + Debug> EtaParam<T> {
    /// Creates the Eta Vector with eta-1 = eta-2 = `i` and eta-3 = eta-4 = eta-5 = eta-6 = 0.
    pub fn zeroed(i: T) -> Self {
        Self::new(
            i.clone(),
            i,
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
        )
    }

    /// Creates the Eta Vector with the listed parameters.
    /// # Panics
    /// Panics if `a` or `b` is less than or equal to 0.
    pub fn new(a: T, b: T, c: T, d: T, e: T, f: T) -> Self {
        assert!(
            a > Zero::zero() && b > Zero::zero(),
            "Eta-1 and eta-2 must be greater than zero! Got {:?} and {:?}",
            a,
            b
        );
        EtaParam(a, b, c, d, e, f)
    }
}

/// Fits an eta-3 spline between 'start' and 'end' using 'eta', according to [this paper](https://ieeexplore.ieee.org/document/4339545/).
pub fn eta_3(start: &MotionState<f64>, end: &MotionState<f64>, eta: &EtaParam<f64>) -> Curve<f64> {
    let mut coeffs = (
        vec![
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
        ],
        vec![
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
        ],
    );
    let ca = start.t.cos();
    let sa = start.t.sin();
    let cb = end.t.cos();
    let sb = end.t.sin();

    // constant terms (u^0)
    coeffs.0[0] = start.x;
    coeffs.1[0] = start.y;
    // linear (u^1)
    coeffs.0[1] = eta.0 * ca;
    coeffs.1[1] = eta.0 * sa;
    // quadratic (u^2)
    coeffs.0[2] = 1. / 2. * eta.2 * ca - 1. / 2. * eta.0.powi(2) * start.k * sa;
    coeffs.1[2] = 1. / 2. * eta.2 * sa + 1. / 2. * eta.0.powi(2) * start.k * ca;
    // cubic (u^3)
    coeffs.0[3] = 1. / 6. * eta.4 * ca
        - 1. / 6. * (eta.0.powi(3) * start.dk + 3. * eta.0 * eta.2 * start.k) * sa;
    coeffs.1[3] = 1. / 6. * eta.4 * sa
        + 1. / 6. * (eta.0.powi(3) * start.dk + 3. * eta.0 * eta.2 * start.k) * ca;
    // quartic (u^4)
    coeffs.0[4] = 35. * (end.x - start.x) - (20. * eta.0 + 5. * eta.2 + 2. / 3. * eta.4) * ca
        + (5. * eta.0.powi(2) * start.k
            + 2. / 3. * eta.0.powi(3) * start.dk
            + 2. * eta.0 * eta.2 * start.k) * sa
        - (15. * eta.1 - 5. / 2. * eta.3 + 1. / 6. * eta.5) * cb
        - (5. / 2. * eta.1.powi(2) * end.k
            - 1. / 6. * eta.1.powi(3) * end.dk
            - 1. / 2. * eta.1 * eta.3 * end.k) * sb;
    coeffs.1[4] = 35. * (end.y - start.y) - (20. * eta.0 + 5. * eta.2 + 2. / 3. * eta.4) * sa
        - (5. * eta.0.powi(2) * start.k
            + 2. / 3. * eta.0.powi(3) * start.dk
            + 2. * eta.0 * eta.2 * start.k) * ca
        - (15. * eta.1 - 5. / 2. * eta.3 + 1. / 6. * eta.5) * sb
        + (5. / 2. * eta.1.powi(2) * end.k
            - 1. / 6. * eta.1.powi(3) * end.dk
            - 1. / 2. * eta.1 * eta.3 * end.k) * cb;
    // quintic (u^5)
    coeffs.0[5] = -84. * (end.x - start.x) + (45. * eta.0 + 10. * eta.2 + eta.4) * ca
        - (10. * eta.0.powi(2) * start.k + eta.0.powi(3) * start.dk + 3. * eta.0 * eta.2 * start.k)
            * sa + (39. * eta.1 - 7. * eta.3 + 1. / 2. * eta.5) * cb
        + (7. * eta.1.powi(2) * end.k
            - 1. / 2. * eta.1.powi(3) * end.dk
            - 3. / 2. * eta.1 * eta.3 * end.k) * sb;
    coeffs.1[5] = -84. * (end.y - start.y) + (45. * eta.0 + 10. * eta.2 + eta.4) * sa
        + (10. * eta.0.powi(2) * start.k + eta.0.powi(3) * start.dk + 3. * eta.0 * eta.2 * start.k)
            * ca + (39. * eta.1 - 7. * eta.3 + 1. / 2. * eta.5) * sb
        - (7. * eta.1.powi(2) * end.k
            - 1. / 2. * eta.1.powi(3) * end.dk
            - 3. / 2. * eta.1 * eta.3 * end.k) * cb;
    // sextic (u^6)
    coeffs.0[6] = 70. * (end.x - start.x) - (36. * eta.0 + 15. / 2. * eta.2 + 2. / 3. * eta.4) * ca
        + (15. / 2. * eta.0.powi(2) * start.k
            + 2. / 3. * eta.0.powi(3) * start.dk
            + 2. * eta.0 * eta.2 * start.k) * sa
        - (34. * eta.1 - 13. / 2. * eta.3 + 1. / 2. * eta.5) * cb
        - (13. / 2. * eta.1.powi(2) * end.k
            - 1. / 2. * eta.1.powi(3) * end.dk
            - 3. / 2. * eta.1 * eta.3 * end.k) * sb;
    coeffs.1[6] = 70. * (end.y - start.y) - (36. * eta.0 + 15. / 2. * eta.2 + 2. / 3. * eta.4) * sa
        - (15. / 2. * eta.0.powi(2) * start.k
            + 2. / 3. * eta.0.powi(3) * start.dk
            + 2. * eta.0 * eta.2 * start.k) * ca
        - (34. * eta.1 - 13. / 2. * eta.3 + 1. / 2. * eta.5) * sb
        + (13. / 2. * eta.1.powi(2) * end.k
            - 1. / 2. * eta.1.powi(3) * end.dk
            - 3. / 2. * eta.1 * eta.3 * end.k) * cb;
    // septic (u^7)
    coeffs.0[7] = -20. * (end.x - start.x) + (10. * eta.0 + 2. * eta.2 + 1. / 6. * eta.4) * ca
        - (2. * eta.0.powi(2) * start.k
            + 1. / 6. * eta.0.powi(3) * start.dk
            + 1. / 2. * eta.0 * eta.2 * start.k) * sa
        + (10. * eta.1 - 2. * eta.3 + 1. / 6. * eta.5) * cb
        + (2. * eta.1.powi(2) * end.k
            - 1. / 6. * eta.1.powi(3) * end.dk
            - 1. / 2. * eta.1 * eta.3 * end.k) * sb;
    coeffs.1[7] = -20. * (end.y - start.y) + (10. * eta.0 + 2. * eta.2 + 1. / 6. * eta.4) * sa
        + (2. * eta.0.powi(2) * start.k
            + 1. / 6. * eta.0.powi(3) * start.dk
            + 1. / 2. * eta.0 * eta.2 * start.k) * ca
        + (10. * eta.1 - 2. * eta.3 + 1. / 6. * eta.5) * sb
        - (2. * eta.1.powi(2) * end.k
            - 1. / 6. * eta.1.powi(3) * end.dk
            - 1. / 2. * eta.1 * eta.3 * end.k) * cb;

    Curve {
        x: Polynomial::new(coeffs.0),
        y: Polynomial::new(coeffs.1),
    }
}

#[cfg(test)]
mod eta_tests {
    extern crate csv;

    use super::*;
    #[test]
    #[should_panic]
    fn panic_eta() {
        EtaParam::new(0, -1, -1, 2, 3, 4);
    }

    #[test]
    fn test() {
        let s = MotionState {
            x: 0.,
            y: 0.,
            t: 0.,
            k: 0.,
            dk: 0.,
        };
        let e = MotionState {
            x: 10.,
            y: 5.,
            t: 3.14,
            k: 0.,
            dk: 0.,
        };

        let eta = EtaParam::new(10., 5., 0., 0., 0., 0.);
        let curve = eta_3(&s, &e, &eta);

        let mut wtr = csv::Writer::from_path("test_out.csv").unwrap();
        wtr.write_record(&["x", "y"]).unwrap();
        let pts = curve.render(100);
        pts.iter().for_each(|p| wtr.serialize(p).unwrap());
    }
}
