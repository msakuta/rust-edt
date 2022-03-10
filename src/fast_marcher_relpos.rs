use super::BoolLike;
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    ops::{Index, IndexMut},
};
use super::Pixel;

/// Shorthand function for EDT using Fast Marching method.
///
/// Fast Marching method is inexact, but much faster algorithm to compute EDT especially for large images.
pub fn edt_fmm<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<Pixel> {
    let mut grid = Grid {
        storage: map
            .iter()
            .map(|b| Pixel {
                val:((b.as_bool() != invert) as usize) as f64,
                relpos: (0, 0),
            })
            .collect::<Vec<_>>(),
        dims: shape,
    };
    let mut fast_marcher = FastMarcher::new_from_map(&grid, shape);

    fast_marcher.evolve(&mut grid);

    grid.storage
}

/// EDT with Fast Marching method with a callback.
///
/// The callback can terminate the process by returning false.
pub fn edt_fmm_cb<T: BoolLike>(
    map: &[T],
    shape: (usize, usize),
    invert: bool,
    callback: impl FnMut(FMMCallbackData) -> bool,
) -> Vec<Pixel> {
    let mut grid = Grid {
        storage: map
            .iter()
            .map(|b| Pixel {
                val:((b.as_bool() != invert) as usize) as f64,
                relpos: (0, 0),
            })
            .collect::<Vec<_>>(),
        dims: shape,
    };
    let mut fast_marcher = FastMarcher::new_from_map(&grid, shape);

    fast_marcher.evolve_cb(&mut grid, callback);

    grid.storage
}

/// A type representing a position in Grid
pub type GridPos = (usize, usize);

#[derive(Clone)]
pub struct Grid {
    pub storage: Vec<Pixel>,
    pub dims: (usize, usize),
}

impl Grid {
    pub fn try_index(&self, pos: GridPos) -> Option<Pixel> {
        let idx = pos.0 * self.dims.0 + pos.1;
        if idx < self.storage.len() {
            Some(self.storage[idx])
        } else {
            None
        }
    }

    pub fn try_index_mut(&mut self, pos: GridPos) -> Option<&mut Pixel> {
        let idx = pos.0 * self.dims.0 + pos.1;
        let storage = &mut self.storage;
        if idx < storage.len() {
            Some(storage.index_mut(idx))
        } else {
            None
        }
    }

    pub fn from_image<T: BoolLike>(image: &[T], dims: (usize, usize)) -> Self {
        Self {
            storage: image.iter().map(|p| Pixel { val: (p.as_bool()) as i32 as f64, relpos: (0, 0)}).collect(),
            dims,
        }
    }

    pub fn find_boundary(&self) -> Vec<GridPos> {
        // let storage = self.storage.as_ref();
        let mut boundary = Vec::new();
        for y in 0..self.dims.1 {
            for x in 0..self.dims.0 {
                if self[(x, y)].val != 0.
                    && (x < 1
                        || self[(x - 1, y)].val == 0.
                        || y < 1
                        || self[(x, y - 1)].val == 0.
                        || self.dims.0 <= x + 1
                        || self[(x + 1, y)].val == 0.
                        || self.dims.1 <= y + 1
                        || self[(x, y + 1)].val == 0.)
                {
                    let pos = (x, y);
                    boundary.push(pos);
                }
            }
        }

        println!("boundary size: {}", boundary.len());

        boundary
    }
}

impl Index<GridPos> for Grid {
    type Output = Pixel;
    fn index(&self, pos: GridPos) -> &Self::Output {
        let idx = pos.1 * self.dims.0 + pos.0;
        self.storage.index(idx)
    }
}

impl IndexMut<GridPos> for Grid {
    fn index_mut(&mut self, pos: GridPos) -> &mut Self::Output {
        let idx = pos.1 * self.dims.0 + pos.0;
        self.storage.index_mut(idx)
    }
}

#[derive(Clone)]
pub(super) struct NextCell {
    pos: GridPos,
    pixel: Pixel,
}

impl PartialEq for NextCell {
    fn eq(&self, other: &Self) -> bool {
        self.pixel.val.eq(&other.pixel.val)
    }
}

impl Eq for NextCell {}

impl PartialOrd for NextCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Reverse(self.pixel.val).partial_cmp(&Reverse(other.pixel.val))
    }
}

impl Ord for NextCell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone)]
pub struct FastMarcher {
    next_cells: BinaryHeap<NextCell>,
    visited: Vec<f64>,
    dims: (usize, usize),
}

impl FastMarcher {
    pub(super) fn new_from_map(grid: &Grid, dims: (usize, usize)) -> Self {
        Self::new(grid.find_boundary().into_iter(), dims)
    }

    pub fn new(next_cells: impl Iterator<Item = GridPos>, dims: (usize, usize)) -> Self {
        let next_cells: BinaryHeap<_> = next_cells
            .map(|gpos| NextCell {
                pos: gpos,
                pixel: Pixel {
                    val: 1.,
                    relpos: (0, 0),
                }
            })
            .collect();
        let mut visited = vec![0.; dims.0 * dims.1];
        for NextCell { pos: (x, y), .. } in &next_cells {
            visited[x + y * dims.0] = 1.;
        }
        Self {
            next_cells,
            visited,
            dims,
        }
    }

    /// Returns whether a pixel has changed; if not, there is no point iterating again
    fn evolve_single(&mut self, grid: &mut Grid) -> bool {
        while let Some(next) = self.next_cells.pop() {
            let x = next.pos.0 as isize;
            let y = next.pos.1 as isize;

            let mut check_neighbor = |x, y| {
                if x < 0 || self.dims.0 as isize <= x || y < 0 || self.dims.1 as isize <= y {
                    return false;
                }
                let get_visited = |x, y| {
                    if x < 0 || self.dims.0 as isize <= x || y < 0 || self.dims.1 as isize <= y {
                        0.
                    } else {
                        self.visited[x as usize + y as usize * self.dims.0]
                    }
                };
                let delta_1d = |p: f64, n: f64| {
                    if p == 0. && n == 0. {
                        None
                    } else if p == 0. {
                        Some(n)
                    } else if n == 0. {
                        Some(p)
                    } else {
                        Some(p.min(n))
                    }
                };
                let u_h = delta_1d(get_visited(x + 1, y), get_visited(x - 1, y));
                let u_v = delta_1d(get_visited(x, y + 1), get_visited(x, y - 1));
                let next_cost = match (u_h, u_v) {
                    (Some(u_h), Some(u_v)) => {
                        let delta = 2. - (u_v - u_h).powf(2.);
                        Pixel {
                            val: if delta < 0. {
                            u_h.min(u_v) + 1.
                        } else {
                            (u_v + u_h + delta.sqrt()) / 2.
                        },
                        relpos: (0, 0),
                        }
                    }
                    (Some(u_h), None) => Pixel {
                        val: u_h + 1.,
                        relpos: (0, 0),
                    },
                    (None, Some(u_v)) => Pixel{
                        val: u_v + 1.,
                        relpos: (0, 0),
                    },
                    _ => panic!("No way"),
                };
                let (x, y) = (x as usize, y as usize);
                let visited = self.visited[x + y * self.dims.0];
                if (visited == 0. || next_cost.val < visited) && grid[(x, y)].val != 0. {
                    self.visited[x + y * self.dims.0] = next_cost.val;
                    let pos = (x, y);
                    grid[pos] = next_cost;
                    self.next_cells.push(NextCell {
                        pos,
                        pixel: next_cost,
                    });
                    true
                } else {
                    false
                }
            };
            let mut f = false;
            f |= check_neighbor(x - 1, y);
            f |= check_neighbor(x, y - 1);
            f |= check_neighbor(x + 1, y);
            f |= check_neighbor(x, y + 1);
            if f {
                return true;
            }
        }
        false
    }
}

#[non_exhaustive]
/// A type that will be given as the argument to the callback with [`crate::edt_fmm_cb`].
///
/// It has `non_exhaustive` attribute so that the library can add more arguments in
/// the future.
pub struct FMMCallbackData<'src> {
    /// The buffer for Fast Marching output in progress.
    pub map: &'src [Pixel],
    /// A dynamically dispatched iterator for positions of next pixels.
    ///
    /// You can examine "expanding wavefront" by iterating this iterator.
    pub next_pixels: &'src mut dyn Iterator<Item = GridPos>,
}

impl FastMarcher {
    pub(super) fn evolve_cb(
        &mut self,
        grid: &mut Grid,
        mut callback: impl FnMut(FMMCallbackData) -> bool,
    ) {
        while self.evolve_single(grid) {
            if !callback(FMMCallbackData {
                map: &grid.storage,
                next_pixels: &mut self.next_cells.iter().map(|nc| nc.pos),
            }) {
                return;
            }
        }
    }

    pub(super) fn evolve(&mut self, grid: &mut Grid) {
        loop {
            if !self.evolve_single(grid) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    fn approx_eq(a: f64, b: f64) {
        if a == 0. && b == 0. {
            return;
        }
        let rel_err = (a - b).abs() / a.abs().max(b.abs());
        assert!(rel_err < 0.2, "a: {}, b: {}", a, b);
    }

    #[test]
    fn test_edt() {
        let map = test_map();
        let str_edt = [
            "0000000000",
            "0001111000",
            "0013443110",
            "0013443100",
            "0001111000",
        ];
        let shape = (map.len() / str_edt.len(), str_edt.len());
        let mut edt = edt_fmm(&map, shape, false);
        for cell in &mut edt {
            cell.val = cell.val.powf(2.);
        }
        eprintln!("edt({:?}):", shape);
        print_2d(&reshape(&edt, shape));
        for (a, b) in edt.iter().zip(parse_edt_str(&str_edt).iter()) {
            approx_eq(a.val, *b);
        }
    }
}
