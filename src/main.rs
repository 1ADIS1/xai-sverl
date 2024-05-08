mod state;
mod tictactoe;

use geng::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    let opts: Opts = clap::Parser::parse();

    let timer = Timer::new();
    let random_policy: tictactoe::Policy = Box::new(|grid: &tictactoe::Grid| {
        let options = grid.empty_positions().count();
        let prob = if options == 0 {
            0.0
        } else {
            (options as f64).recip()
        };
        tictactoe::Grid::from_fn(|pos| match grid.get(pos) {
            Some(tictactoe::Cell::Empty) => prob,
            _ => 0.0,
        })
    });
    let shapley = tictactoe::Grid::new().shapley(&random_policy);
    println!("{:?}", shapley);
    println!("calc took {}ms", timer.elapsed().as_secs_f64() * 1000.0);
    // return;

    logger::init();
    geng::setup_panic_handler();

    let mut options = geng::ContextOptions::default();
    options.window.title = "XAI".into();
    options.with_cli(&opts.geng);
    Geng::run_with(&options, |geng| async move {
        let state = state::State::new(&geng);
        geng.run_state(state).await;
    });
}
