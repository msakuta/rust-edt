use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    ops::{Index, IndexMut},
    time::Instant,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A type representing a position in Grid
pub struct GridPos {
    pub row: usize,
    pub col: usize,
}

pub(super) struct Grid {
    pub storage: Vec<f64>,
    pub dims: (usize, usize),
}

impl Grid {
    pub(super) fn find_boundary(&self) -> Vec<GridPos> {
        // let storage = self.storage.as_ref();
        let mut boundary = Vec::new();
        for y in 0..self.dims.1 {
            for x in 0..self.dims.0 {
                if self[GridPos { col: x, row: y }] != 0.
                    && (x < 1
                        || self[GridPos { col: x - 1, row: y }] == 0.
                        || y < 1
                        || self[GridPos { col: x, row: y - 1 }] == 0.
                        || self.dims.0 <= x + 1
                        || self[GridPos { col: x + 1, row: y }] == 0.
                        || self.dims.1 <= y + 1
                        || self[GridPos { col: x, row: y + 1 }] == 0.)
                {
                    let pos = GridPos { col: x, row: y };
                    boundary.push(pos);
                }
            }
        }

        println!("boundary size: {}", boundary.len());

        boundary
    }
}

impl Index<GridPos> for Grid {
    type Output = f64;
    fn index(&self, pos: GridPos) -> &Self::Output {
        let idx = pos.row * self.dims.0 + pos.col;
        self.storage.index(idx)
    }
}

impl IndexMut<GridPos> for Grid {
    fn index_mut(&mut self, pos: GridPos) -> &mut Self::Output {
        let idx = pos.row * self.dims.0 + pos.col;
        self.storage.index_mut(idx)
    }
}

#[derive(Clone)]
pub(super) struct NextCell {
    pos: GridPos,
    cost: f32,
}

impl PartialEq for NextCell {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}

impl Eq for NextCell {}

impl PartialOrd for NextCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Reverse(self.cost).partial_cmp(&Reverse(other.cost))
    }
}

impl Ord for NextCell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone)]
pub(super) struct FastMarcher {
    next_cells: BinaryHeap<NextCell>,
    visited: Vec<f32>,
    dims: (usize, usize),
}

impl FastMarcher {
    pub(super) fn new_from_map(grid: &Grid, dims: (usize, usize)) -> Self {
        Self::new(grid.find_boundary().into_iter(), dims)
    }

    pub(super) fn new(next_cells: impl Iterator<Item = GridPos>, dims: (usize, usize)) -> Self {
        let next_cells: BinaryHeap<_> = next_cells
            .map(|gpos| NextCell {
                pos: gpos,
                cost: 1.,
            })
            .collect();
        let mut visited = vec![0.; dims.0 * dims.1];
        for NextCell {
            pos: GridPos { col: x, row: y },
            ..
        } in &next_cells
        {
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
            let x = next.pos.col as isize;
            let y = next.pos.row as isize;

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
                let delta_1d = |p: f32, n: f32| {
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
                        if delta < 0. {
                            u_h.min(u_v) + 1.
                        } else {
                            (u_v + u_h + delta.sqrt()) / 2.
                        }
                    }
                    (Some(u_h), None) => u_h + 1.,
                    (None, Some(u_v)) => u_v + 1.,
                    _ => panic!("No way"),
                };
                let (x, y) = (x as usize, y as usize);
                let visited = self.visited[x + y * self.dims.0];
                if (visited == 0. || next_cost < visited) && grid[GridPos { col: x, row: y }] != 0.
                {
                    self.visited[x + y * self.dims.0] = next_cost;
                    let pos = GridPos { col: x, row: y };
                    let cost = (next_cost) as f64;
                    grid[pos] = cost;
                    self.next_cells.push(NextCell {
                        pos,
                        cost: next_cost,
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
/// A type that will be given as the argument to the callback with [`edt_fmm_cb`].
///
/// It has `non_exhaustive` attribute so that the library can add more arguments in
/// the future.
pub struct FMMCallbackData<'src> {
    /// The buffer for Fast Marching output in progress.
    pub map: &'src [f64],
    /// A dynamically dispatched iterator for positions of next pixels.
    ///
    /// You can examine "expanding wavefront" by iterating this iterator.
    pub next_pixels: &'src mut dyn Iterator<Item = GridPos>,
}

impl FastMarcher {
    pub(super) fn evolve(
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

    pub(super) fn evolve_steps(&mut self, grid: &mut Grid, steps: usize) {
        let start = Instant::now();
        let mut evolved = false;
        for _ in 0..steps {
            if !self.evolve_single(grid) {
                break;
            }
            evolved = true;
        }
        if evolved {
            println!(
                "FastMarcher::evolve: next_cells: {}, visited: {}, time: {}",
                self.next_cells.len(),
                self.visited.iter().filter(|p| 0. < **p).count(),
                start.elapsed().as_nanos() as f64 * 1e-9
            );
        }
    }
}
