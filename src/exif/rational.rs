use little_exif::rational::{iR64, uR64};
use num_rational::Ratio;

const VMAX: f64 = 2147483647.0;

const POWS_OF_TEN: [i32; 10] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
];

pub fn approx_frac(value: f64) -> Option<(f64, i32, i32)> {
    let (value, minus) = if value < 0.0 { (-value, true) } else { (value, false) };
    if value > VMAX { return None; }

    let mut best: (i32, i32) = (1, 1);
    let mut best_score = f64::INFINITY;

    for &d in &POWS_OF_TEN {
        let n_f = (value * d as f64).round();
        if !(n_f > 0.0 && n_f <= VMAX) { continue; }

        let n = n_f as i32;
        let score = calc_score(value, n, d);
        if score < best_score {
            best_score = score;
            best = (n, d)
        }
    }

    match Ratio::<i32>::approximate_float(value) {
        Some(r) => {
            let (n, d) = (*r.numer(), *r.denom());
            let score = calc_score(value, n, d);
            if score < best_score {
                // best_score = score;
                best = (n, d);
            }
        }
        None => {}
    };
    let (n, d) = if minus { (best.0 as i32 * -1, best.1) } else { (best.0 as i32, best.1) };
    Some((n as f64 / d as f64, n, d))
}

fn calc_score(v: f64, n: i32, d: i32) -> f64 {
    let digits_score = num_digits(n) + num_digits(d);
    let err = (v - (n as f64 / d as f64)).abs();
    let epsilon = 1e-12;
    let error_score = (err + epsilon).log10();

    1.0 * digits_score as f64 + 5.0 * error_score
}

fn num_digits(n: i32) -> usize {
    if n == 0 {
        1
    } else {
        (n as f64).log10().floor() as usize + 1
    }
}

pub trait ExifRational {
    fn to_string(&self) -> String;
    fn new(nominator: i32, denominator: i32) -> Self;
}

impl ExifRational for iR64 {
    fn to_string(&self) -> String {
        format!("{}/{}", self.nominator, self.denominator)
    }
    fn new(nominator: i32, denominator: i32) -> Self {
        Self { nominator, denominator }
    }
}

impl ExifRational for uR64 {
    fn to_string(&self) -> String {
        format!("{}/{}", self.nominator, self.denominator)
    }
    fn new(nominator: i32, denominator: i32) -> Self {
        Self { nominator: nominator as u32, denominator : denominator as u32}
    }
}