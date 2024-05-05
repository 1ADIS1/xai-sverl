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

    // let timer = Timer::new();
    // let mut counts = Vec::new();
    // let mut sum = 0;
    // for observation in tictactoe::Grid::new().all_subsets() {
    //     let states = observation.possible_states();
    //     counts.push(states.len());
    //     sum += states.len();
    // }
    // println!("calc took {}ms", timer.elapsed().as_secs_f64() * 1000.0);
    // println!("observations: {}", counts.len());
    // println!("sum: {}", sum);
    // println!("{:?}", counts);
    // return;

    logger::init();
    geng::setup_panic_handler();

    let mut options = geng::ContextOptions::default();
    options.with_cli(&opts.geng);
    Geng::run("XAI", |geng| async move {
        let state = state::State::new(&geng);
        geng.run_state(state).await;
    });
}
