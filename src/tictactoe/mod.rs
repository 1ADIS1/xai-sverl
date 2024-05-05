use geng::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    X,
    O,
}

pub type Coord = usize;

#[derive(Debug, Clone)]
pub struct Grid {
    pub cells: [[Cell; 3]; 3],
}

impl Grid {
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
