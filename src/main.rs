mod state;
mod tictactoe;

use geng::prelude::*;

#[derive(clap::Parser)]
struct Opts {
    #[clap(subcommand)]
    command: Option<Command>,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

#[derive(clap::Subcommand)]
enum Command {
    Test,
}

fn main() {
    let opts: Opts = clap::Parser::parse();

    if let Some(command) = &opts.command {
        match command {
            Command::Test => {
                println!("\nRandom policy");
                let mut timer = Timer::new();
                let shapley = tictactoe::Grid::new().shapley(&mut tictactoe::policy_random());
                println!("{:?}", shapley);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                println!("\nMinimax policy");
                let mut timer = Timer::new();
                let mut cache = std::collections::BTreeMap::new();
                let mut policy = tictactoe::policy_minimax_cached(None, &mut cache);
                let shapley = tictactoe::Grid::new().shapley(&mut policy);
                println!("{:?}", shapley);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                return;
            }
        }
    }

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
