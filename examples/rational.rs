use exif_edit::exif::rational::approx_frac;

fn main() {
    let value = 0.125;
    match approx_frac(value) {
        Some((v, n, d)) => {
            println!("{} {} {}", v, n, d);
        }
        None => {}
    }
}