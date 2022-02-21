// mod save_img;

use clap::Parser;
use edt::{edt, edt_fmm, edt_fmm_cb, FMMCallbackData};
use image::{GenericImageView, ImageBuffer, Luma, Rgb};
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(default_value = "Rust_Logo.png", help = "File name to apply EDT.")]
    file_name: String,
    #[clap(short, long, help = "Use Fast Marching method")]
    fast_marching: bool,
    #[clap(
        short,
        long,
        help = "Steps in FMM to produce images in progress. Each image is prefixed by step number.
Warning! don't put too small number, or it will produce lots of images!"
    )]
    progress_steps: Option<usize>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let img = image::open(args.file_name).unwrap();
    let dims = img.dimensions();
    println!("dimensions {:?}, color: {:?}", dims, img.color());

    let img2 = img.into_luma8();
    let flat_samples = img2.as_flat_samples();

    let slice = flat_samples.image_slice().unwrap();

    println!("len {}", slice.len());

    let start = Instant::now();

    let mut i = 0;

    let edt_f64 = if args.fast_marching {
        if let Some(progress_steps) = args.progress_steps {
            edt_fmm_cb(
                &slice,
                (dims.0 as usize, dims.1 as usize),
                true,
                |FMMCallbackData {
                     map: edt_f64,
                     next_pixels,
                     ..
                 }| {
                    if i % progress_steps == 0 {
                        let max_value =
                            edt_f64.iter().map(|p| *p).reduce(f64::max).unwrap().max(1.);
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

                        edt_img.save(&format!("edt{}.png", i)).unwrap();
                    }
                    i += 1;

                    true
                },
            )
        } else {
            edt_fmm(&slice, (dims.0 as usize, dims.1 as usize), true)
        }
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

    edt_img.save("edt.png").unwrap();

    Ok(())
}
