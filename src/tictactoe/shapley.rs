use super::*;

#[derive(Debug, Clone)]
pub struct Observation {
    pub positions: Vec<vec2<Coord>>,
    pub grid: Grid,
}

impl Grid<Cell> {
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
