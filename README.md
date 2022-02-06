# Rust-edt

An implementation of 2D EDT ([Euclidian distance transform](https://en.wikipedia.org/wiki/Distance_transform)) with Saito's algorithm in pure Rust

There is also [another crate](https://crates.io/crates/distance-transform) that implements EDT implementation,
but I would like to reinvent a wheel that has these advantages.

* No dependencies (except example codes)
* Intuitive to use (accepts a boolean slice and a shape)

Performance was not the priority, but I would like to explore more optimizations.

![Rust-logo](Rust_logo.png)
![Rust-logo-edt](Rust_logo_edt.png)

## Usage

Add dependency

```toml
[dependencies]
edt = "0.1.0"
```

Prepare a binary image as a flattened vec.
This library assumes that the input is a flat vec for 2d image.

```rust
let mut vec: Vec<u8> = vec![/*...*/];
```

Call edt with given shape

```rust
use edt::edt;

let edt_image = edt(&vec, (32, 32));
```

Save to a file if you want.
The code below normalizes the value with maximum value to 8 bytes grayscale image.

```rust
use image::{ImageBuffer, Luma};

let max_value = edt_image.iter().map(|p| *p).reduce(f64::max).unwrap();
let edt_img = edt_image
    .iter()
    .map(|p| (*p / max_value * 255.) as u8)
    .collect();

let edt_img: ImageBuffer<Luma<u8>, Vec<u8>> =
    ImageBuffer::from_vec(dims.0, dims.1, edt_img).unwrap();

// Write the contents of this image to the Writer in PNG format.
edt_img.save("edt.png").unwrap();
```

## Literature

doi=10.1.1.66.2644

https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.66.2644&rep=rep1&type=pdf

Section 7.7
