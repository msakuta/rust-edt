use edt::edt;
use image::{ImageBuffer, Luma};

fn main() {
    const SIZE: usize = 512;
    const HALFSIZE: usize = SIZE / 2;

    let mut map = vec![false; SIZE * SIZE];

    for i in 0..SIZE {
        for j in 0..SIZE {
            let dx = j - HALFSIZE;
            let dy = i - HALFSIZE;
            map[j + i * SIZE] = dx * dx + dy * dy < HALFSIZE * HALFSIZE;
        }
    }

    let edt_f64 = edt(&map, (SIZE, SIZE));

    let max_value = edt_f64.iter().map(|p| *p).reduce(f64::max).unwrap();
    let edt_img = edt_f64
        .iter()
        .map(|p| (*p / max_value * 255.) as u8)
        .collect();

    let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_vec(SIZE as u32, SIZE as u32, edt_img).unwrap();

    // Write the contents of this image to the Writer in PNG format.
    edt_img.save("edt.png").unwrap();
}
