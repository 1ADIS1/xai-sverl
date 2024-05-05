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

    logger::init();
    geng::setup_panic_handler();

    let mut options = geng::ContextOptions::default();
    options.with_cli(&opts.geng);
    Geng::run("XAI", |geng| async move {
        let state = state::State::new(&geng);
        geng.run_state(state).await;
    });
}
