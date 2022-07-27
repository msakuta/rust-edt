use super::BoolLike;
use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    ops::{Index, IndexMut},
};

/// Shorthand function for EDT using Fast Marching method.
///
/// Fast Marching method is inexact, but much faster algorithm to compute EDT especially for large images.
pub fn edt_fmm<T: BoolLike>(map: &[T], shape: (usize, usize), invert: bool) -> Vec<PixelAbs> {
    let mut grid = Grid {
        storage: map
            .iter()
            .enumerate()
            .map(|(i, b)| PixelAbs {
                val: ((b.as_bool() != invert) as usize) as f64,
                abspos: (i % shape.0, i / shape.0),
            })
            .collect::<Vec<_>>(),
        dims: shape,
    };
    let mut fast_marcher = FastMarcher::new_from_map(&grid, shape);

    fast_marcher.evolve(&mut grid, usize::MAX);

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
) -> Vec<PixelAbs> {
    let mut grid = Grid {
        storage: map
            .iter()
            .enumerate()
            .map(|(i, b)| PixelAbs {
                val: ((b.as_bool() != invert) as usize) as f64,
                abspos: (i % shape.0, i / shape.0),
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
    pub storage: Vec<PixelAbs>,
    pub dims: (usize, usize),
}

impl Grid {
    pub fn try_index(&self, pos: GridPos) -> Option<PixelAbs> {
        let idx = pos.0 * self.dims.0 + pos.1;
        if idx < self.storage.len() {
            Some(self.storage[idx])
        } else {
            None
        }
    }

    pub fn try_index_mut(&mut self, pos: GridPos) -> Option<&mut PixelAbs> {
        let idx = pos.0 * self.dims.0 + pos.1;
        let storage = &mut self.storage;
        if idx < storage.len() {
            Some(storage.index_mut(idx))
        } else {
            None
        }
    }

    pub fn invert(&mut self, pos: GridPos) {
        if let Some(pixel) = self.try_index(pos) {
            self.try_index_mut(pos)
                .map(|pixel_mut| pixel_mut.val = (pixel.val == 0.) as i32 as f64);
        }
    }

    pub fn from_image<T: BoolLike>(image: &[T], dims: (usize, usize)) -> Self {
        Self {
            storage: image
                .iter()
                .enumerate()
                .map(|(i, p)| PixelAbs {
                    val: (p.as_bool()) as i32 as f64,
                    abspos: (i % dims.0, i / dims.0),
                })
                .collect(),
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
    type Output = PixelAbs;
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

/// Custom type that returns EDT value and relative position to the closest foreground pixel.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PixelAbs {
    pub val: f64,
    pub abspos: (usize, usize),
}

#[derive(Clone)]
pub(super) struct NextCell {
    pos: GridPos,
    pixel: PixelAbs,
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
    visited: Vec<PixelAbs>,
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
                pixel: PixelAbs {
                    val: 1e-4,
                    abspos: gpos,
                },
            })
            .collect();
        let mut visited = vec![PixelAbs::default(); dims.0 * dims.1];
        for NextCell { pos: (x, y), .. } in &next_cells {
            visited[x + y * dims.0].val = 1.;
            visited[x + y * dims.0].abspos = (*x, *y);
        }
        Self {
            next_cells,
            visited,
            dims,
        }
    }

    /// Returns whether a pixel has changed; if not, there is no point iterating again
    pub fn evolve_single(&mut self, grid: &mut Grid, speed_map: Option<&Grid>) -> bool {
        while let Some(next) = self.next_cells.pop() {
            let x = next.pos.0 as isize;
            let y = next.pos.1 as isize;

            let delta_1d = |p: PixelAbs, n: PixelAbs| {
                if p.val == 0. && n.val == 0. {
                    None
                } else if p.val == 0. {
                    Some(n)
                } else if n.val == 0. {
                    Some(p)
                } else {
                    Some(if p.val < n.val { p } else { n })
                }
            };

            let mut freeze_neighbor = |x, y| {
                if x < 0 || self.dims.0 as isize <= x || y < 0 || self.dims.1 as isize <= y {
                    return false;
                }
                let get_visited = |dx, dy| {
                    let (x, y) = (x + dx as isize, y + dy as isize);
                    if x < 0 || self.dims.0 as isize <= x || y < 0 || self.dims.1 as isize <= y {
                        PixelAbs::default()
                    } else {
                        let neighbor = self.visited[x as usize + y as usize * self.dims.0];
                        neighbor
                        // PixelAbs {
                        //     val: neighbor.val,
                        //     abspos: (x as usize + 1, y as usize),
                        // }
                    }
                };
                let u_h = delta_1d(get_visited(1, 0), get_visited(-1, 0));
                let u_v = delta_1d(get_visited(0, 1), get_visited(0, -1));
                let speed = speed_map
                    .map(|map| map[(x as usize, y as usize)].val)
                    .unwrap_or(1.);
                let frozen_value = match (u_h, u_v) {
                    (Some(u_h), Some(u_v)) => {
                        let delta = speed * 2. - (u_v.val - u_h.val).powf(2.);
                        if delta < 0. {
                            if u_h.val < u_v.val {
                                PixelAbs {
                                    val: u_h.val + speed.sqrt(),
                                    abspos: u_h.abspos,
                                }
                            } else {
                                PixelAbs {
                                    val: u_v.val + speed.sqrt(),
                                    abspos: u_v.abspos,
                                }
                            }
                        } else {
                            PixelAbs {
                                val: (u_v.val + u_h.val + delta.sqrt()) / 2.,
                                abspos: if u_v.val < u_h.val {
                                    u_v.abspos
                                } else {
                                    u_h.abspos
                                },
                            }
                        }
                    }
                    (Some(u_h), None) => PixelAbs {
                        val: u_h.val + speed.sqrt(),
                        abspos: u_h.abspos,
                    },
                    (None, Some(u_v)) => PixelAbs {
                        val: u_v.val + speed.sqrt(),
                        abspos: u_v.abspos,
                    },
                    _ => return false,
                };
                let (x, y) = (x as usize, y as usize);
                self.visited[x + y * self.dims.0] = frozen_value;
                let pos = (x, y);
                grid[pos] = frozen_value;
                true
            };

            freeze_neighbor(x, y);

            let mut check_neighbor = |x, y| {
                if x < 0 || self.dims.0 as isize <= x || y < 0 || self.dims.1 as isize <= y {
                    return false;
                }
                let get_visited = |dx, dy| {
                    let (x, y) = (x + dx as isize, y + dy as isize);
                    if x < 0 || self.dims.0 as isize <= x || y < 0 || self.dims.1 as isize <= y {
                        PixelAbs::default()
                    } else {
                        let neighbor = self.visited[x as usize + y as usize * self.dims.0];
                        neighbor
                        // PixelAbs {
                        //     val: neighbor.val,
                        //     abspos: (x as usize + 1, y as usize),
                        // }
                    }
                };
                let u_h = delta_1d(get_visited(1, 0), get_visited(-1, 0));
                let u_v = delta_1d(get_visited(0, 1), get_visited(0, -1));
                let speed = speed_map
                    .map(|map| map[(x as usize, y as usize)].val)
                    .unwrap_or(1.);
                let next_pixel = match (u_h, u_v) {
                    (Some(u_h), Some(u_v)) => {
                        let delta = speed * 2. - (u_v.val - u_h.val).powf(2.);
                        if delta < 0. {
                            if u_h.val < u_v.val {
                                PixelAbs {
                                    val: u_h.val + speed.sqrt(),
                                    abspos: u_h.abspos,
                                }
                            } else {
                                PixelAbs {
                                    val: u_v.val + speed.sqrt(),
                                    abspos: u_v.abspos,
                                }
                            }
                        } else {
                            PixelAbs {
                                val: (u_v.val + u_h.val + delta.sqrt()) / 2.,
                                abspos: if u_v.val < u_h.val {
                                    u_v.abspos
                                } else {
                                    u_h.abspos
                                },
                            }
                        }
                    }
                    (Some(u_h), None) => PixelAbs {
                        val: u_h.val + speed.sqrt(),
                        abspos: u_h.abspos,
                    },
                    (None, Some(u_v)) => PixelAbs {
                        val: u_v.val + speed.sqrt(),
                        abspos: u_v.abspos,
                    },
                    _ => panic!("No way"),
                };
                let (x, y) = (x as usize, y as usize);
                let visited = self.visited[x + y * self.dims.0];
                if (visited.val == 0. || next_pixel.val < visited.val) && grid[(x, y)].val != 0. {
                    self.visited[x + y * self.dims.0] = next_pixel;
                    let pos = (x, y);
                    // grid[pos] = next_pixel;
                    self.next_cells.push(NextCell {
                        pos,
                        pixel: next_pixel,
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

    pub fn iter_cells(&self) -> impl Iterator<Item = &GridPos> {
        self.next_cells.iter().map(|cell| &cell.pos)
    }
}

#[non_exhaustive]
/// A type that will be given as the argument to the callback with [`crate::edt_fmm_cb`].
///
/// It has `non_exhaustive` attribute so that the library can add more arguments in
/// the future.
pub struct FMMCallbackData<'src> {
    /// The buffer for Fast Marching output in progress.
    pub map: &'src [PixelAbs],
    /// A dynamically dispatched iterator for positions of next pixels.
    ///
    /// You can examine "expanding wavefront" by iterating this iterator.
    pub next_pixels: &'src mut dyn Iterator<Item = GridPos>,
    pub visited: &'src [PixelAbs],
}

impl FastMarcher {
    /// A version of `evolve` that the user can provide with a callback to
    /// terminate the fast marching with any condition. If `callback` returns false,
    /// the search will stop.
    /// You could resume search by calling `evolve` again.
    ///
    /// Returns whether the callback has requested to stop.
    /// If it returns true, there can be more processable pixels.
    pub fn evolve_cb(
        &mut self,
        grid: &mut Grid,
        mut callback: impl FnMut(FMMCallbackData) -> bool,
    ) -> bool {
        while self.evolve_single(grid, None) {
            if !callback(FMMCallbackData {
                map: &grid.storage,
                next_pixels: &mut self.next_cells.iter().map(|nc| nc.pos),
                visited: &self.visited,
            }) {
                return true;
            }
        }
        false
    }

    /// A customizable version of `evolve_cb` that you can use speed field
    pub fn evolve_speed_cb(
        &mut self,
        grid: &mut Grid,
        speed_map: &Grid,
        mut callback: impl FnMut(FMMCallbackData) -> bool,
    ) -> bool {
        while self.evolve_single(grid, Some(speed_map)) {
            if !callback(FMMCallbackData {
                map: &grid.storage,
                next_pixels: &mut self.next_cells.iter().map(|nc| nc.pos),
                visited: &self.visited,
            }) {
                return true;
            }
        }
        false
    }

    /// Returns whether Fast Marching Method has terminated within steps or it can
    /// make progress if called again.
    pub fn evolve(&mut self, grid: &mut Grid, steps: usize) -> bool {
        for _ in 0..steps {
            if !self.evolve_single(grid, None) {
                return false;
            }
        }
        true
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

    impl PrintDist for PixelAbs {
        fn print(&self) {
            print!("({:.1} {:2} {:2})", self.val, self.abspos.0, self.abspos.1);
        }
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
