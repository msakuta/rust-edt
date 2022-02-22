use edt::{edt, edt_fmm};
use image::{ImageBuffer, Luma};
use std::{env, time::Instant};

fn main() {
    const SIZE: usize = 512;

    let size = env::args()
        .skip(1)
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(SIZE);
    let half_size = size / 2;
    let quater_size = (size / 4) as isize;

    let use_fmm = env::args()
        .skip(2)
        .next()
        .map(|s| s == "-e")
        .unwrap_or(false);

    let mut map = vec![false; size * size];

    for i in 0..size {
        for j in 0..size {
            let dx = j as isize - half_size as isize;
            let dy = i as isize - half_size as isize;
            map[j + i * size] = dx.abs() < quater_size || dy.abs() < quater_size;
        }
    }

    let start = Instant::now();

    let edt_f64 = if use_fmm { edt_fmm } else { edt }(&map, (size, size), false);

    let duration = start.elapsed().as_micros();
    println!("time: {:?}ms", duration as f64 / 1e3);
    let max_value = edt_f64.iter().map(|p| *p).reduce(f64::max).unwrap();
    let edt_img = edt_f64
        .iter()
        .map(|p| (*p / max_value * 255.) as u8)
        .collect();

    let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_vec(size as u32, size as u32, edt_img).unwrap();

    // Write the contents of this image to the Writer in PNG format.
    edt_img.save("edt.png").unwrap();
}
