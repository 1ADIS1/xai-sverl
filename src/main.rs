mod controls;
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

#[derive(geng::asset::Load, Serialize, Deserialize)]
#[load(serde = "toml")]
struct Config {
    palette: Palette,
}

#[derive(Serialize, Deserialize)]
struct Palette {
    background: Rgba<f32>,
    text: Rgba<f32>,
    grid: Rgba<f32>,
    eval_negative: Rgba<f32>,
    eval_positive: Rgba<f32>,
    button_background: Rgba<f32>,
    button_background_hover: Rgba<f32>,
    button_border: Rgba<f32>,
    button_border_active: Rgba<f32>,
}

fn main() {
    let opts: Opts = clap::Parser::parse();

    if let Some(command) = &opts.command {
        match command {
            Command::Test => {
                println!("\nRandom policy");
                let mut timer = Timer::new();
                let mut policy = tictactoe::policy_random();
                let shapley = tictactoe::Grid::new().shapley(&mut policy);
                println!("shapley: {:?}", shapley);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                let sverl = tictactoe::Grid::new().sverl(false, 0.5, &mut policy);
                println!("sverl local: {:?}", sverl);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                let sverl = tictactoe::Grid::new().sverl(true, 0.5, &mut policy);
                println!("sverl global: {:?}", sverl);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                println!("\nMinimax policy");
                let mut timer = Timer::new();
                let mut cache = std::collections::BTreeMap::new();
                let mut policy = tictactoe::policy_minimax_cached(None, &mut cache);
                let shapley = tictactoe::Grid::new().shapley(&mut policy);
                println!("shapley: {:?}", shapley);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                let sverl = tictactoe::Grid::new().sverl(false, 0.5, &mut policy);
                println!("sverl local: {:?}", sverl);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                let sverl = tictactoe::Grid::new().sverl(true, 0.5, &mut policy);
                println!("sverl global: {:?}", sverl);
                println!("calc took {}ms", timer.tick().as_secs_f64() * 1000.0);

                return;
            }
        }
    }

    logger::init_with({
        let mut builder = logger::builder();
        builder.filter_level(log::LevelFilter::Debug);
        builder
    })
    .expect("failed to initialize logger");
    geng::setup_panic_handler();

    let mut options = geng::ContextOptions::default();
    options.window.title = "XAI".into();
    options.with_cli(&opts.geng);
    Geng::run_with(&options, |geng| async move {
        let manager = geng.asset_manager();
        let config: Config =
            geng::asset::Load::load(manager, &run_dir().join("assets").join("config.toml"), &())
                .await
                .expect("failed to load config");

        let state = state::State::new(&geng, config);
        geng.run_state(state).await;
    });
}
