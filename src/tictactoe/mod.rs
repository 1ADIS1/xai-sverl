mod policy;
mod shapley;
mod sverl;

pub use self::policy::*;

use geng::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    Empty,
    X,
    O,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn next(&self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl From<Player> for Tile {
    fn from(value: Player) -> Self {
        match value {
            Player::X => Tile::X,
            Player::O => Tile::O,
        }
    }
}

pub type Coord = usize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grid<T = Tile> {
    pub cells: [[T; 3]; 3],
}

impl<T> Grid<T> {
    pub fn from_fn(mut f: impl FnMut(vec2<Coord>) -> T) -> Self {
        Self {
            cells: std::array::from_fn(|y| std::array::from_fn(|x| f(vec2(x, y)))),
        }
    }

    pub fn bounds(&self) -> Aabb2<Coord> {
        Aabb2::ZERO.extend_positive(vec2(self.cells[0].len(), self.cells.len()))
    }

    pub fn positions(&self) -> impl Iterator<Item = vec2<Coord>> + '_ {
        self.bounds().points()
    }

    pub fn set(&mut self, position: vec2<Coord>, value: T) {
        if let Some(target) = self
            .cells
            .get_mut(position.y)
            .and_then(|row| row.get_mut(position.x))
        {
            *target = value;
        }
    }

    pub fn get(&self, position: vec2<Coord>) -> Option<&T> {
        self.cells
            .get(position.y)
            .and_then(|row| row.get(position.x))
    }
}

impl Grid<Tile> {
    pub fn new() -> Self {
        Self {
            cells: [[Tile::Empty; 3]; 3],
        }
    }

    pub fn check(&self, pos: vec2<Coord>) -> bool {
        self.get(pos)
            .map_or(false, |tile| matches!(tile, Tile::Empty))
    }

    pub fn empty_positions(&self) -> impl Iterator<Item = vec2<Coord>> + '_ {
        self.positions()
            .filter(|&pos| matches!(self.get(pos), Some(Tile::Empty)))
    }

    pub fn current_player(&self) -> Option<Player> {
        if self.winner().is_some() || self.empty_positions().next().is_none() {
            return None;
        }

        let mut count_x = 0;
        let mut count_o = 0;
        for pos in self.positions() {
            match self.get(pos) {
                Some(Tile::X) => count_x += 1,
                Some(Tile::O) => count_o += 1,
                _ => {}
            }
        }

        if count_x > count_o {
            Some(Player::O)
        } else {
            Some(Player::X)
        }
    }

    pub fn winner(&self) -> Option<Player> {
        // horizontal
        for row in &self.cells {
            if *row == [Tile::X; 3] {
                return Some(Player::X);
            }
            if *row == [Tile::O; 3] {
                return Some(Player::O);
            }
        }

        // vertical
        for x in self.bounds().min.x..self.bounds().max.x {
            let mut winner_x = true;
            let mut winner_o = true;
            for y in self.bounds().min.y..self.bounds().max.y {
                if self.get(vec2(x, y)) != Some(&Tile::X) {
                    winner_x = false;
                }
                if self.get(vec2(x, y)) != Some(&Tile::O) {
                    winner_o = false;
                }
            }
            if winner_x {
                return Some(Player::X);
            }
            if winner_o {
                return Some(Player::O);
            }
        }

        // diagonal
        let mut winner_main_x = true;
        let mut winner_main_o = true;
        let mut winner_sec_x = true;
        let mut winner_sec_o = true;
        for x in self.bounds().min.x..self.bounds().max.x {
            if self.get(vec2(x, x)) != Some(&Tile::X) {
                winner_main_x = false;
            }
            if self.get(vec2(x, x)) != Some(&Tile::O) {
                winner_main_o = false;
            }

            let y = self.bounds().max.y.saturating_sub(x).saturating_sub(1);
            if self.get(vec2(x, y)) != Some(&Tile::X) {
                winner_sec_x = false;
            }
            if self.get(vec2(x, y)) != Some(&Tile::O) {
                winner_sec_o = false;
            }
        }
        if winner_main_x || winner_sec_x {
            return Some(Player::X);
        }
        if winner_main_o || winner_sec_o {
            return Some(Player::O);
        }

        None
    }

    pub fn reward(&self, player: Player) -> f64 {
        match self.winner() {
            None => 0.0,
            Some(winner) => {
                if winner == player {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

impl Grid<f64> {
    pub fn zero() -> Self {
        Self {
            cells: [[0.0; 3]; 3],
        }
    }

    pub fn sum(&self) -> f64 {
        self.cells.iter().flatten().copied().sum()
    }

    pub fn normalize(&self) -> Self {
        let sum = self.sum();
        if sum == 0.0 {
            return self.clone();
        }
        Self::from_fn(|pos| *self.get(pos).unwrap() / sum)
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

impl MulAssign<Self> for Grid<f64> {
    fn mul_assign(&mut self, rhs: Self) {
        for pos in self.positions().collect::<Vec<_>>() {
            let value = self.get(pos).unwrap() * rhs.get(pos).unwrap();
            self.set(pos, value);
        }
    }
}
