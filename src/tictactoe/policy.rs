use super::*;

pub type Policy = Box<dyn Fn(&Grid) -> Grid<f64>>;
pub type Action = vec2<Coord>;

pub fn policy_minimax(depth: Option<usize>) -> Policy {
    Box::new(move |grid| {
        let (action, _) = minimax(grid, Player::X, depth, 0);
        Grid::from_fn(|pos| if pos == action { 1.0 } else { 0.0 })
    })
}

fn minimax(grid: &Grid, player: Player, limit: Option<usize>, depth: usize) -> (Action, f64) {
    grid.empty_positions()
        .map(|action| {
            let mut grid = grid.clone();
            grid.set(action, player.into());

            let value = evaluate(&grid, player);
            if value != 0.0 || limit.map_or(false, |limit| depth >= limit) {
                // Game finished
                return (action, value);
            }

            // Recursion
            minimax(&grid, player.next(), limit, depth + 1)
        })
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap_or((vec2::ZERO, 0.0))
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
