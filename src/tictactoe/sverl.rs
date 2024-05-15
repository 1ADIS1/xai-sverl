use super::*;

use std::collections::BTreeMap;

impl Grid<Tile> {
    pub fn sverl(&self, global: bool, gamma: f64, policy: &mut Policy) -> Grid<f64> {
        let Some(player) = self.current_player() else {
            return Grid::zero();
        };

        let mut cache = BTreeMap::new();
        let values = self.shapley_with_value(|feature, observation| {
            let states = observation.possible_states();
            let prob = (states.len() as f64).recip();
            let mut first = states
                .into_iter()
                .map(|state| policy(&state))
                .fold(Grid::zero(), Grid::add);
            first *= prob;
            // `first` here is pi_c, an approximation of the policy given limited knowledge

            let mut result = Grid::zero();
            for pos in self.empty_positions() {
                let prob = match first.get(pos) {
                    Some(&p) if p > 0.0 => p,
                    _ => continue,
                };

                let mut grid = self.clone();
                grid.set(pos, player.into());
                let value = if global {
                    let mut policy = Box::new(|state: &Grid| {
                        let mut observation = state.full_observation();
                        let sub = observation.subtract(feature);
                        assert!(sub, "Full observation does not have the feature");

                        let states = observation.possible_states();
                        let prob = (states.len() as f64).recip();
                        let mut res = states
                            .into_iter()
                            .map(|state| policy(&state))
                            .fold(Grid::zero(), Grid::add);
                        res *= prob; // pi_c, an approximation of the policy given limited knowledge
                        res
                    }) as Policy;
                    grid.predict(&mut cache, gamma, &mut policy)
                } else {
                    grid.predict(&mut cache, gamma, policy)
                };
                result.set(pos, prob * value);
            }
            result
        });
        Grid::from_fn(|pos| values.get(pos).unwrap().sum())
    }

    fn predict(
        &self,
        cache: &mut BTreeMap<Grid<Tile>, f64>,
        gamma: f64,
        policy: &mut Policy,
    ) -> f64 {
        if let Some(&cached) = cache.get(self) {
            return cached;
        }

        let Some(player) = self.current_player() else {
            return 0.0;
        };

        let mut result = 0.0;
        let weights = policy(self);
        for pos in self.empty_positions() {
            let prob = match weights.get(pos) {
                Some(&p) if p > 0.0 => p,
                _ => continue,
            };

            let mut grid = self.clone();
            grid.set(pos, player.into());
            result += prob * (grid.reward(player) + gamma * (grid.predict(cache, gamma, policy)));
        }

        cache.insert(self.clone(), result);
        result
    }
}
