use clap::Parser;
use edt::{edt, edt_fmm, edt_relpos};
use image::{ImageBuffer, Luma, Rgb};
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "512",
        help = "Size of the synthesized edt image"
    )]
    size: usize,
    #[clap(short, long, help = "Use Fast Marching method")]
    fast_marching: bool,
    #[clap(short, long, help = "Make difference between exact and Fast Marching")]
    diff: bool,
    #[clap(
        short,
        long,
        help = "Use relative vector to the nearest background. The output image will be rgb image wher red is edt intensity, blue is horizontal relative position and green is vertical relative position"
    )]
    use_relpos: bool,
}

fn main() {
    let args = Args::parse();
    let size = args.size;
    let half_size = size / 2;
    let quater_size = (size / 4) as isize;

    let mut map = vec![false; size * size];

    for i in 0..size {
        for j in 0..size {
            let dx = j as isize - half_size as isize;
            let dy = i as isize - half_size as isize;
            map[j + i * size] = dx.abs() < quater_size || dy.abs() < quater_size;
        }
    }

    (if args.use_relpos {
        main_edt_relpos
    } else {
        main_edt
    })(args, &map)
}

fn main_edt(args: Args, map: &[bool]) {
    let use_fmm = args.fast_marching;
    let size = args.size;

    let start = Instant::now();

    let edt_f64 = if use_fmm { edt_fmm } else { edt }(map, (size, size), false);

    let duration = start.elapsed().as_micros();
    println!("time: {:?}ms", duration as f64 / 1e3);
    let max_value = edt_f64.iter().map(|p| *p).reduce(f64::max).unwrap();
    let edt_img = edt_f64
        .iter()
        .map(|p| (*p / max_value * 255.) as u8)
        .collect();

    let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_vec(size as u32, size as u32, edt_img).unwrap();

    edt_img.save("edt.png").unwrap();
}

fn main_edt_relpos(args: Args, map: &[bool]) {
    let size = args.size;

    let start = Instant::now();

    let edt_f64 = edt_relpos(&map, (size, size), false);

    let duration = start.elapsed().as_micros();
    println!("time: {:?}ms", duration as f64 / 1e3);
    let max_value = edt_f64.iter().map(|p| p.val).reduce(f64::max).unwrap();
    let max_relpos = edt_f64
        .iter()
        .map(|p| p.relpos[0].abs().max(p.relpos[1].abs()))
        .max()
        .unwrap();
    let edt_img = edt_f64
        .iter()
        .map(|p| {
            [
                (p.val / max_value * 255.) as u8,
                (p.relpos[0] * 127 / max_relpos) as u8 + 127,
                (p.relpos[1] * 127 / max_relpos) as u8 + 127,
            ]
        })
        .flatten()
        .collect();

    let edt_img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_vec(size as u32, size as u32, edt_img).unwrap();

    edt_img.save("edt.png").unwrap();
}
