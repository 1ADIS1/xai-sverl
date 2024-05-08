mod shapley;

use geng::prelude::*;

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
