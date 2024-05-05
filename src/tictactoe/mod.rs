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

    pub fn all_subsets(&self) -> Vec<Observation> {
        let positions: Vec<_> = self.positions().collect();
        powerset(&positions)
            .into_iter()
            .map(|positions| Observation {
                positions,
                grid: self.clone(),
            })
            .collect()
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
}

fn powerset<T>(s: &[T]) -> Vec<Vec<T>>
where
    T: Clone,
{
    (0..2usize.pow(s.len() as u32))
        .map(|i| {
            s.iter()
                .enumerate()
                .filter(|&(t, _)| (i >> t) % 2 == 1)
                .map(|(_, element)| element.clone())
                .collect()
        })
        .collect()
}
