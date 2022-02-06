use image::{GenericImageView, ImageBuffer, Luma};
use rust_edt::edt;
use std::env;

fn main() -> std::io::Result<()> {
    let file_name = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "sample.png".to_string());

    let img = image::open(file_name).unwrap();
    let dims = img.dimensions();
    println!("dimensions {:?}, color: {:?}", dims, img.color());

    let img2 = img.into_luma8();

    let mut vec = vec![];
    vec.extend(
        img2.as_flat_samples()
            .image_slice()
            .unwrap()
            .iter()
            .map(|b| *b == 0),
    );

    println!("len {}", vec.len());

    let edt_f64 = edt(&vec, (dims.0 as usize, dims.1 as usize));
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