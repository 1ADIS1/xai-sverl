use geng::prelude::*;

pub type Action = vec2<Coord>;
pub type Policy = Box<dyn Fn(&Grid) -> Grid<f64>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    X,
    O,
}

pub type Coord = usize;

#[derive(Debug, Clone)]
pub struct Grid<T = Cell> {
    pub cells: [[T; 3]; 3],
}

impl<T> Grid<T> {
    pub fn from_fn(f: impl Fn(vec2<Coord>) -> T) -> Self {
        Self {
            cells: std::array::from_fn(|y| std::array::from_fn(|x| f(vec2(x, y)))),
        }
    }
}

impl Grid<Cell> {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; 3]; 3],
        }
    }

    pub fn bounds(&self) -> Aabb2<Coord> {
        Aabb2::ZERO.extend_positive(vec2(self.cells[0].len(), self.cells.len()))
    }

    pub fn positions(&self) -> impl Iterator<Item = vec2<Coord>> + '_ {
        self.bounds().points()
    }

    pub fn empty_positions(&self) -> impl Iterator<Item = vec2<Coord>> + '_ {
        self.positions()
            .filter(|&pos| matches!(self.get(pos), Some(Cell::Empty)))
    }

    pub fn get(&self, position: vec2<Coord>) -> Option<Cell> {
        self.cells
            .get(position.y)
            .and_then(|row| row.get(position.x))
            .copied()
    }

    pub fn set(&mut self, position: vec2<Coord>, cell: Cell) {
        if let Some(target) = self
            .cells
            .get_mut(position.y)
            .and_then(|row| row.get_mut(position.x))
        {
            *target = cell;
        }
    }

    pub fn full_observation(&self) -> Observation {
        Observation {
            positions: self.positions().collect(),
            grid: self.clone(),
        }
    }

    pub fn all_subsets(&self) -> Vec<Observation> {
        let positions: Vec<_> = self.positions().collect();
        powerset(&positions)
            .into_iter()
            .skip(1) // Skip the set itself
            .map(|positions| Observation {
                positions,
                grid: self.clone(),
            })
            .collect()
    }

    pub fn shapley(&self, policy: &Policy) -> Grid<f64> {
        let base_observation = self.full_observation();
        let base_value = base_observation.value(policy);
        let subsets = self.all_subsets();
        let scale = (subsets.len() as f64).recip();

        let mut result = subsets
            .into_iter()
            .map(|observation| {
                let mut value = observation.value(policy) - base_value.clone();
                value *= factorial(observation.positions.len()) as f64
                    * factorial(base_observation.positions.len() - observation.positions.len() - 1)
                        as f64;
                value
            })
            .fold(Grid::zero(), Grid::add);

        result *= scale;
        result
    }
}

impl Grid<f64> {
    pub fn zero() -> Self {
        Self {
            cells: [[0.0; 3]; 3],
        }
    }
}

impl Add<Self> for Grid<f64> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_fn(|vec2(x, y)| self.cells[y][x] + rhs.cells[y][x])
    }
}

impl Sub<Self> for Grid<f64> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_fn(|vec2(x, y)| self.cells[y][x] - rhs.cells[y][x])
    }
}

impl MulAssign<f64> for Grid<f64> {
    fn mul_assign(&mut self, rhs: f64) {
        for cell in self.cells.iter_mut().flatten() {
            *cell *= rhs
        }
    }
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub positions: Vec<vec2<Coord>>,
    pub grid: Grid,
}

impl Observation {
    pub fn possible_states(&self) -> Vec<Grid> {
        let hidden: Vec<_> = self
            .grid
            .positions()
            .filter(|pos| !self.positions.contains(pos))
            .collect();

        (0..3usize.pow(hidden.len() as u32))
            .map(|i| {
                let mut grid = self.grid.clone();
                for (pos, cell) in hidden.iter().enumerate().map(|(t, &pos)| {
                    let cell = match (i / 3_usize.pow(t as u32)) % 3 {
                        0 => Cell::Empty,
                        1 => Cell::X,
                        2 => Cell::O,
                        _ => unreachable!(),
                    };
                    (pos, cell)
                }) {
                    grid.set(pos, cell);
                }
                grid
            })
            .collect()
    }

    pub fn value(&self, policy: &Policy) -> Grid<f64> {
        let states = self.possible_states();
        let prob = (states.len() as f64).recip();

        let mut result = states
            .into_iter()
            .map(|state| policy(&state))
            .fold(Grid::zero(), Grid::add);
        result *= prob;
        result
    }
}

fn powerset<T>(s: &[T]) -> Vec<Vec<T>>
where
    T: Clone,
{
    (0..2usize.pow(s.len() as u32))
        .map(|i| {
            s.iter()
                .enumerate()
                .filter(|&(t, _)| (i >> t) % 2 == 0)
                .map(|(_, element)| element.clone())
                .collect()
        })
        .collect()
}

fn factorial(x: usize) -> usize {
    (2..=x).fold(1, usize::mul)
}
