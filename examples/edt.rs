// mod save_img;

use edt::{edt, edt_fmm, edt_fmm_cb, FMMCallbackData};
use image::{GenericImageView, ImageBuffer, Luma, Rgb};
use std::{env, time::Instant};

fn main() -> std::io::Result<()> {
    let file_name = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "Rust_logo.png".to_string());

    let use_fmm = env::args()
        .skip(2)
        .next()
        .map(|s| s == "-e")
        .unwrap_or(false);

    let img = image::open(file_name).unwrap();
    let dims = img.dimensions();
    println!("dimensions {:?}, color: {:?}", dims, img.color());

    let img2 = img.into_luma8();
    let flat_samples = img2.as_flat_samples();

    let slice = flat_samples.image_slice().unwrap();

    println!("len {}", slice.len());

    let start = Instant::now();

    let mut i = 0;

    let edt_f64 = if use_fmm {
        edt_fmm_cb(
            &slice,
            (dims.0 as usize, dims.1 as usize),
            true,
            |FMMCallbackData {
                 map: edt_f64,
                 next_pixels,
                 ..
             }| {
                if i % 500 == 0 {
                    let max_value = edt_f64.iter().map(|p| *p).reduce(f64::max).unwrap().max(1.);
                    let edt_u8: Vec<_> = edt_f64
                        .iter()
                        .map(|p| (*p / max_value * 255.) as u8)
                        .collect();

                    let mut edt_img: ImageBuffer<Rgb<u8>, Vec<_>> =
                        ImageBuffer::new(dims.0, dims.1);
                    for (x, y, pixel) in edt_img.enumerate_pixels_mut() {
                        let luma = edt_u8[(x + y * dims.0) as usize];
                        *pixel = Rgb([luma, luma, luma]);
                    }

                    for pixel in next_pixels {
                        edt_img[(pixel.col as u32, pixel.row as u32)] = Rgb([255, 0, 0]);
                    }

                    // Write the contents of this image to the Writer in PNG format.
                    edt_img.save(&format!("edt{}.png", i)).unwrap();
                }
                i += 1;

                true
            },
        )
    } else {
        edt(&slice, (dims.0 as usize, dims.1 as usize), true)
    };

    let duration = start.elapsed().as_micros();
    println!("time: {:?}ms", duration as f64 / 1e3);
    let max_value = edt_f64.iter().map(|p| *p).reduce(f64::max).unwrap();
    let edt_img = edt_f64
        .iter()
        .map(|p| (*p / max_value * 255.) as u8)
        .collect();

    let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_vec(dims.0, dims.1, edt_img).unwrap();

    // Write the contents of this image to the Writer in PNG format.
    edt_img.save("edt.png").unwrap();

    Ok(())
}
