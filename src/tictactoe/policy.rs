use std::collections::BTreeMap;

use super::*;

pub type Policy<'a> = Box<dyn FnMut(&Grid) -> Grid<f64> + 'a>;
pub type Action = vec2<Coord>;

fn choose_action(probs: Grid<f64>) -> Action {
    let mut rng: f64 = thread_rng().gen();
    let mut last = vec2::ZERO;
    for pos in probs.positions() {
        let &p = probs.get(pos).unwrap();
        if p <= 0.0 {
            continue;
        }
        rng -= p;
        if rng <= 0.0 {
            return pos;
        }
        last = pos;
    }
    // it's theoretically unreachable, but could get here with floating point imprecision or invalid input
    last
}

pub fn random_action(grid: &Grid) -> Action {
    let probs = policy_random()(grid);
    choose_action(probs)
}

pub fn policy_random() -> Policy<'static> {
    Box::new(|grid: &Grid| {
        let options = grid.empty_positions().count();
        let prob = if options == 0 {
            0.0
        } else {
            (options as f64).recip()
        };
        Grid::from_fn(|pos| match grid.get(pos) {
            Some(Tile::Empty) => prob,
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
    cache: &mut BTreeMap<Grid, Grid<f64>>,
) -> Policy<'_> {
    Box::new(move |grid| {
        let Some(player) = grid.current_player() else {
            return Grid::zero();
        };
        minimax_probability(grid, cache, player, depth)
    })
}

pub fn minimax_action(
    grid: &Grid,
    cache: &mut BTreeMap<Grid, Grid<f64>>,
    player: Player,
    limit: Option<usize>,
) -> (Action, f64) {
    let probs = minimax_probability(grid, cache, player, limit);
    let values = minimax(grid, cache, player, limit, 0);
    let action = choose_action(probs);
    let mut value = *values.get(action).unwrap();
    if value.abs() <= 1e-5 {
        value = 0.0;
    }
    (action, value)
}

pub fn minimax_probability(
    grid: &Grid,
    cache: &mut BTreeMap<Grid, Grid<f64>>,
    player: Player,
    limit: Option<usize>,
) -> Grid<f64> {
    let values = minimax(grid, cache, player, limit, 0);
    let max_value = values
        .positions()
        .filter(|&pos| grid.check(pos))
        .map(|pos| *values.get(pos).unwrap())
        .max_by_key(|&v| r64(v))
        .unwrap_or(0.0);
    Grid::from_fn(|pos| {
        if grid.check(pos) && values.get(pos).unwrap().approx_eq(&max_value) {
            1.0
        } else {
            0.0
        }
    })
    .normalize()
}

pub fn minimax(
    grid: &Grid,
    cache: &mut BTreeMap<Grid, Grid<f64>>,
    player: Player,
    limit: Option<usize>,
    depth: usize,
) -> Grid<f64> {
    // log::debug!(
    //     "[depth {}] minimax for player {:?} at {:?}",
    //     depth,
    //     player,
    //     grid
    // );

    if let Some(cached) = cache.get(grid) {
        // log::debug!("[depth {}] cached: {:?}", depth, cached);
        return cached.clone();
    }

    let res = Grid::from_fn(|action| {
        if !grid.check(action) {
            return 0.0;
        }

        let mut grid = grid.clone();
        grid.set(action, player.into());

        // log::debug!("[depth {}] evaluating move {:?}", depth, action);

        let value = grid.reward(player);
        if value != 0.0 || limit.map_or(false, |limit| depth >= limit) {
            // Game finished
            // log::debug!("[depth {}] game ended {:.2}", depth, value);
            return value / (depth + 1) as f64;
        }

        // Recursion
        let deep = minimax(&grid, cache, player.next(), limit, depth + 1);
        let value = deep
            .cells
            .into_iter()
            .flatten()
            .map(r64)
            .max()
            .unwrap()
            .raw();
        -value
    });
    cache.insert(grid.clone(), res.clone());
    // log::debug!("[depth {}] result: {:?}", depth, res);
    res
}
