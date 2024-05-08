use std::collections::BTreeMap;

use super::*;

pub type Policy<'a> = Box<dyn FnMut(&Grid) -> Grid<f64> + 'a>;
pub type Action = vec2<Coord>;

pub fn policy_random() -> Policy<'static> {
    Box::new(|grid: &Grid| {
        let options = grid.empty_positions().count();
        let prob = if options == 0 {
            0.0
        } else {
            (options as f64).recip()
        };
        Grid::from_fn(|pos| match grid.get(pos) {
            Some(Cell::Empty) => prob,
            _ => 0.0,
        })
    })
}

// pub fn policy_minimax(depth: Option<usize>) -> Policy<'static> {
//     Box::new(move |grid| {
//         let (action, _) = minimax(grid, &mut BTreeMap::new(), Player::X, depth, 0);
//         Grid::from_fn(|pos| if pos == action { 1.0 } else { 0.0 })
//     })
// }

pub fn policy_minimax_cached(
    depth: Option<usize>,
    cache: &mut BTreeMap<Grid, (Action, f64)>,
) -> Policy<'_> {
    Box::new(move |grid| {
        let (action, _) = minimax(grid, cache, Player::X, depth, 0);
        Grid::from_fn(|pos| if pos == action { 1.0 } else { 0.0 })
    })
}

pub fn minimax(
    grid: &Grid,
    cache: &mut BTreeMap<Grid, (Action, f64)>,
    player: Player,
    limit: Option<usize>,
    depth: usize,
) -> (Action, f64) {
    if let Some(cached) = cache.get(grid) {
        return *cached;
    }

    let res = grid
        .empty_positions()
        .map(|action| {
            let mut grid = grid.clone();
            grid.set(action, player.into());

            let value = evaluate(&grid, player);
            if value != 0.0 || limit.map_or(false, |limit| depth >= limit) {
                // Game finished
                return (action, value);
            }

            // Recursion
            let (action, value) = minimax(&grid, cache, player.next(), limit, depth + 1);
            (action, -value)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap_or((vec2(999, 999), -999.0));
    cache.insert(grid.clone(), res);
    res
}

fn evaluate(grid: &Grid, player: Player) -> f64 {
    match grid.winner() {
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
