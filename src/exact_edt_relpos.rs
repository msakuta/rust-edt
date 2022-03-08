use super::BoolLike;
use std::cmp::{Ordering, PartialOrd};

/// Custom type that returns EDT value and relative position to the closest forward pixel.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Pixel {
    pub val: f64,
    pub relpos: [i32; 2],
}

impl PartialOrd for Pixel {
    fn partial_cmp(&self, other: &Pixel) -> Option<Ordering> {
        self.val.partial_cmp(&other.val)
    }
}

/// Produce an EDT from binary image.
///
/// The returned vec has the same size as the input slice, containing
/// computed EDT.
///
/// It assumes zero pixels are obstacles. If you want to invert the logic,
/// put `true` to the third argument.
pub fn edt<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<Pixel> {
    let mut ret = edt_sq(map, shape, invert);
    for pixel in &mut ret {
        pixel.val = pixel.val.sqrt();
    }
    ret
}

/// Squared EDT of a given image.
///
/// The interface is equivalent to [`edt`], but it returns squared EDT.
///
/// It is more efficient if you only need squared edt, because you wouldn't need to compute square root.
pub fn edt_sq<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<Pixel> {
    let horz_edt = horizontal_edt(map, shape, invert);

    let vertical_scan = |x, y: usize| {
        let total_edt = (0..shape.1).map(|y2| {
            let horz_p: &Pixel = &horz_edt[x + y2 * shape.0];
            let horz_val = horz_p.val;
            Pixel {
                val: (y2 as f64 - y as f64).powf(2.) + horz_val.powf(2.),
                relpos: [horz_p.relpos[0], y as i32 - y2 as i32],
            }
        });
        let vmin = total_edt
            .reduce(|a, b| if a.val < b.val { a } else { b })
            .unwrap();

        if (y as f64).powf(2.) < vmin.val {
            Pixel {
                val: (y as f64).powf(2.),
                relpos: [0, -(y as i32)],
            }
        } else if ((shape.1 - y) as f64).powf(2.) < vmin.val {
            Pixel {
                val: ((shape.1 - y) as f64).powf(2.),
                relpos: [0, shape.1 as i32 - y as i32],
            }
        } else {
            vmin
        }
    };

    let mut ret = vec![Pixel::default(); shape.0 * shape.1];

    for x in 0..shape.0 {
        for y in 0..shape.1 {
            ret[x + y * shape.0] = vertical_scan(x, y);
        }
    }

    ret
}

fn horizontal_edt<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<Pixel> {
    let mut horz_edt = map
        .iter()
        .map(|b| Pixel {
            val: (((b.as_bool() != invert) as usize) * map.len()) as f64,
            relpos: [0, 0],
        })
        .collect::<Vec<_>>();

    let scan = |x, y, min_p: &mut Pixel, horz_edt: &mut Vec<Pixel>, rel_x| {
        let p: &mut Pixel = &mut horz_edt[x + y * shape.0];
        let next = min_p.val + 1.;
        if next < p.val {
            p.val = next;
            p.relpos[0] = min_p.relpos[0] + rel_x;
        }
        *min_p = *p;
    };

    for y in 0..shape.1 {
        let mut min_val = Pixel {
            val: 0.,
            relpos: [0, 0],
        };
        for x in 0..shape.0 {
            scan(x, y, &mut min_val, &mut horz_edt, -1);
        }
        min_val = Pixel {
            val: 0.,
            relpos: [0, 0],
        };
        for x in (0..shape.0).rev() {
            scan(x, y, &mut min_val, &mut horz_edt, 1);
        }
    }

    horz_edt
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;
    use itertools::Itertools;

    impl PrintDist for Pixel {
        fn print(&self) {
            print!(
                "({:.1} {:2} {:2})",
                self.val, self.relpos[0], self.relpos[1]
            );
        }
    }

    fn parse_packed(chunks: impl Iterator<Item = char>) -> i32 {
        let x = chunks.collect::<String>();
        let x: i32 = x.trim().parse().unwrap();
        x
    }

    pub(super) fn parse_edt_str_pixel(s: &[&str], xs: &[&str], ys: &[&str]) -> Vec<Pixel> {
        flatten(
            s.iter()
                .zip(xs.iter().zip(ys))
                .map(|(s, (xs, ys))| {
                    s.chars()
                        .zip(
                            xs.chars()
                                .chunks(2)
                                .into_iter()
                                .zip(ys.chars().chunks(2).into_iter()),
                        )
                        .map(|(c, (x, y))| Pixel {
                            val: if c != 'f' {
                                (c as u8 - '0' as u8) as f64
                            } else {
                                15.
                            },
                            relpos: [parse_packed(x), parse_packed(y)],
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn test_edt() {
        let map = test_map();
        let str_edt = [
            "0000000000",
            "0001111000",
            "0012442110",
            "0012442100",
            "0001111000",
        ];
        let str_xs = [
            " 0 0 0 0 0 0 0 0 0 0",
            " 0 0 0-1 0 0 1 0 0 0",
            " 0 0-1-1 0 0 1 0 0 0",
            " 0 0 0-1 0 0 1 0 0 0",
            " 0 0 0-1 0 0 1 0 0 0",
        ];
        let str_ys = [
            " 0 0 0 0 0 0 0 0 0 0",
            " 0 0 0 0 1 1 0 0 0 0",
            " 0 0 0 1 2 2 1 1-1 0",
            " 0 0-1-1 2 2-1-1 0 0",
            " 0 0 0 0 1 1 0 0 0 0",
        ];
        let shape = (map.len() / str_edt.len(), str_edt.len());
        let edt = edt_sq(&map, shape, false);
        eprintln!("edt({:?}) size={}:", shape, std::mem::size_of_val(&str_xs));
        print_2d(&reshape(&edt, shape));
        let expected = parse_edt_str_pixel(&str_edt, &str_xs, &str_ys);
        print_2d(&reshape(&expected, shape));
        for (i, (a, b)) in edt.iter().zip(expected.iter()).enumerate() {
            assert_eq!(a, b, "{}", i);
        }
    }
}
