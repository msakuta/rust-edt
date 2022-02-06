# Rust-edt

An implementation of 2D EDT ([Euclidian distance transform](https://en.wikipedia.org/wiki/Distance_transform)) with Saito's algorithm in pure Rust

There is also [another crate](https://crates.io/crates/distance-transform) that implements EDT implementation,
but I would like to reinvent a wheel that has these advantages:

* No dependencies (except example codes)
* Intuitive to use (accepts a boolean slice and a shape)

Performance was not the priority, but I would like to explore more optimizations.

![Rust-logo](https://github.com/msakuta/rust-edt/blob/master/Rust_logo.png)
![Rust-logo-edt](https://github.com/msakuta/rust-edt/blob/master/Rust_logo_edt.png)

EDT is the basis of many algorithms, but it is hard to find in a general purpose image processing library,
probably because the algorithm is not trivial to implement efficiently.
This crate provides an implementation of EDT in fairly efficient algorithm presented in the literature.

The algorithm used in this crate (Saito's algorithm) is O(n^3), where n is the number of pixels along one direction.
Naive computation of EDT would be O(n^4), so it is certainly better than that, but there is also fast-marching based
algorithm that is O(n^2).

## Usage

Add dependency

```toml
[dependencies]
edt = "0.1.0"
```

Prepare a binary image as a flattened vec.
This library assumes that the input is a flat vec for 2d image.

```rust
let mut vec: Vec<bool> = vec![/*...*/];
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


### 2D Euclidean Distance Transform Algorithms: A Comparative Survey

This paper is a great summary of this field of research.

doi=10.1.1.66.2644

https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.66.2644&rep=rep1&type=pdf

Section 7.7


### Saito and Toriwaki [1994] (Original paper)

https://www.cs.jhu.edu/~misha/ReadingSeminar/Papers/Saito94.pdf