// mod save_img;

use clap::Parser;
use edt::{edt, edt_fmm, edt_fmm_cb, edt_relpos, FMMCallbackData};
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
        help = "Steps in FMM to produce images in progress. Each image is postfixed by step number.
For example, if steps=1000, edt0.png, edt1000.png, ... will be produced.
Warning! don't put too small number, or it will produce lots of images!"
    )]
    progress_steps: Option<usize>,
    #[clap(short, long, help = "Make difference between exact and Fast Marching")]
    diff: bool,
    #[clap(
        short,
        long,
        help = "Use relative vector to the nearest background. The output image will be rgb image wher red is edt intensity, blue is horizontal relative position and green is vertical relative position"
    )]
    use_relpos: bool,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let img = image::open(&args.file_name).unwrap();
    let dims = img.dimensions();
    println!("dimensions {:?}, color: {:?}", dims, img.color());

    let img2 = img.into_luma8();
    let flat_samples = img2.as_flat_samples();

    let slice = flat_samples.image_slice().unwrap();

    println!("len {}", slice.len());

    (if args.use_relpos {
        main_edt_relpos
    } else {
        main_edt
    })(args, slice, dims)
}

fn main_edt(args: Args, slice: &[u8], dims: (u32, u32)) -> std::io::Result<()> {
    let start = Instant::now();

    let mut i = 0;

    let edt_f64 = if args.diff {
        let fmm = edt_fmm(&slice, (dims.0 as usize, dims.1 as usize), true);
        let exact = edt_relpos(&slice, (dims.0 as usize, dims.1 as usize), true);
        let result: Vec<_> = fmm
            .into_iter()
            .zip(exact.into_iter())
            .map(|(a, b)| a - b.val)
            .collect();
        println!(
            "Max diff: {}",
            result
                .iter()
                .map(|p| p.abs())
                .reduce(f64::max)
                .unwrap()
                .max(1.)
        );
        result
    } else if args.fast_marching {
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
                            edt_img[(pixel.0 as u32, pixel.1 as u32)] = Rgb([255, 0, 0]);
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
    let max_value = edt_f64.iter().map(|p| p.abs()).reduce(f64::max).unwrap();
    if args.diff {
        let edt_img = edt_f64
            .iter()
            .map(|p| {
                [
                    ((-p).max(0.) / max_value * 255.) as u8,
                    0,
                    (p.max(0.) / max_value * 255.) as u8,
                ]
            })
            .flatten()
            .collect();

        let edt_img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_vec(dims.0, dims.1, edt_img).unwrap();

        edt_img.save("edt.png").unwrap();
    } else {
        let edt_img = edt_f64
            .iter()
            .map(|p| (p / max_value * 255.) as u8)
            .collect();

        let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_vec(dims.0, dims.1, edt_img).unwrap();

        edt_img.save("edt.png").unwrap();
    };

    Ok(())
}

fn main_edt_relpos(_args: Args, slice: &[u8], dims: (u32, u32)) -> std::io::Result<()> {
    let edt_f64 = edt_relpos(&slice, (dims.0 as usize, dims.1 as usize), true);

    let edt_img = edt_f64
        .iter()
        .map(|p| {
            [
                0, //(p.val / max_value * 255.) as u8,
                (p.relpos[0] + 127) as u8,
                (p.relpos[1] + 127) as u8,
            ]
        })
        .flatten()
        .collect();

    let edt_img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_vec(dims.0, dims.1, edt_img).unwrap();

    edt_img.save("edt.png").unwrap();

    Ok(())
}
