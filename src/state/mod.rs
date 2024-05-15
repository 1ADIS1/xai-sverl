use crate::{
    tictactoe::{Tile, *},
    Config,
};

use std::collections::BTreeMap;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct State {
    geng: Geng,
    config: Config,
    framebuffer_size: vec2<usize>,
    ui: Ui,
    camera: Camera2d,
    policy: Policy,
    method: Method,
    sverl_global: bool,

    model: Grid,
    minimax_cache: BTreeMap<Grid, Grid<f64>>,
    shapley_values: Option<Grid<Grid<f64>>>,
    sverl_values_local: Option<Grid<f64>>,
    sverl_values_global: Option<Grid<f64>>,
}

#[derive(Debug, Clone, Copy)]
enum Policy {
    Random,
    Minimax,
}

#[derive(Debug, Clone, Copy)]
enum Method {
    Shapley,
    Sverl { global: bool },
}

pub struct Ui {
    font_size: f32,
    cursor_pos: vec2<f32>,
    policy_random: Aabb2<f32>,
    policy_minimax: Aabb2<f32>,
    method_shapley: Aabb2<f32>,
    method_sverl: Aabb2<f32>,
    method_sverl_global: Aabb2<f32>,
    board_reset: Aabb2<f32>,
    board_policy_turn: Aabb2<f32>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            font_size: 1.0,
            cursor_pos: vec2::ZERO,
            policy_random: Aabb2::ZERO,
            policy_minimax: Aabb2::ZERO,
            method_shapley: Aabb2::ZERO,
            method_sverl: Aabb2::ZERO,
            method_sverl_global: Aabb2::ZERO,
            board_reset: Aabb2::ZERO,
            board_policy_turn: Aabb2::ZERO,
        }
    }

    pub fn layout(&mut self, framebuffer_size: vec2<f32>) {
        let font_size = framebuffer_size.x.min(framebuffer_size.y) * 0.03;
        let font_size = font_size.max(20.0);
        self.font_size = font_size;

        let button_size = vec2(7.0, 2.0) * font_size;
        let button = Aabb2::ZERO.extend_positive(button_size);

        let pos = framebuffer_size * vec2(0.1, 0.7);
        let offset = vec2(0.0, button.height() + font_size * 0.5);
        self.policy_random = button.translate(pos);
        self.policy_minimax = button.translate(pos - offset);
        self.method_shapley = button.translate(pos - offset * 3.0);
        self.method_sverl = button.translate(pos - offset * 4.0);
        self.board_reset = button.translate(pos - offset * 6.0);
        self.board_policy_turn = button.translate(pos - offset * 7.0);

        let tickbox_size = vec2::splat(1.5) * font_size;
        let tickbox = Aabb2::ZERO.extend_symmetric(tickbox_size / 2.0);
        let pos = geng_utils::layout::aabb_pos(self.method_sverl, vec2(1.0, 0.5));
        self.method_sverl_global = tickbox
            .translate(pos)
            .translate(vec2(font_size + tickbox.width() / 2.0, 0.0));
    }
}

impl State {
    pub fn new(geng: &Geng, config: Config) -> State {
        let mut state = State {
            geng: geng.clone(),
            config,
            framebuffer_size: vec2(1, 1),
            ui: Ui::new(),
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            policy: Policy::Minimax,
            method: Method::Sverl { global: false },
            sverl_global: false,

            model: Grid::new(),
            minimax_cache: BTreeMap::new(),
            shapley_values: None,
            sverl_values_local: None,
            sverl_values_global: None,
        };
        state.update_values(true);
        state
    }

    fn draw_x(&self, pos: vec2<usize>, transparency: f32, framebuffer: &mut ugli::Framebuffer) {
        let ratio = 0.7;
        let aabb =
            Aabb2::point(pos.as_f32() + vec2(0.5, 0.5)).extend_symmetric(vec2(ratio, ratio) / 2.0);

        let mut color = self.config.palette.grid;
        color.a *= transparency;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Segment::new(Segment(aabb.bottom_left(), aabb.top_right()), 0.1, color),
        );
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Segment::new(Segment(aabb.top_left(), aabb.bottom_right()), 0.1, color),
        );
    }

    fn draw_o(&self, pos: vec2<usize>, transparency: f32, framebuffer: &mut ugli::Framebuffer) {
        let ratio = 0.9;
        let aabb =
            Aabb2::point(pos.as_f32() + vec2(0.5, 0.5)).extend_symmetric(vec2(ratio, ratio) / 2.0);

        let mut color = self.config.palette.grid;
        color.a *= transparency;
        self.geng.draw2d().draw2d(
            framebuffer,
            &self.camera,
            &draw2d::Ellipse::circle_with_cut(
                aabb.center(),
                aabb.width() / 2.0 * ratio * 0.9,
                aabb.width() / 2.0 * ratio,
                color,
            ),
        );
    }

    fn update_values(&mut self, reset: bool) {
        if reset {
            self.shapley_values = None;
            self.sverl_values_local = None;
            self.sverl_values_global = None;
        }

        log::debug!("updating shapley and sverl values");
        let mut policy = match self.policy {
            Policy::Random => policy_random(),
            Policy::Minimax => policy_minimax_cached(None, &mut self.minimax_cache),
        };

        let mut timer = Timer::new();
        match self.method {
            Method::Shapley => {
                if self.shapley_values.is_some() {
                    log::debug!("shapley values cached");
                    return;
                }
                self.shapley_values = Some(self.model.shapley(&mut policy));
                log::debug!(
                    "updated shapley values in {:.3}s",
                    timer.tick().as_secs_f64()
                );
            }
            Method::Sverl { global } => {
                if global {
                    if self.sverl_values_global.is_some() {
                        log::debug!("sverl global values cached");
                        return;
                    }
                    self.sverl_values_global = Some(self.model.sverl(true, 0.9, &mut policy));
                    log::debug!(
                        "updated sverl global values in {:.3}s",
                        timer.tick().as_secs_f64()
                    );
                } else {
                    if self.sverl_values_local.is_some() {
                        log::debug!("sverl local values cached");
                        return;
                    }
                    self.sverl_values_local = Some(self.model.sverl(false, 0.9, &mut policy));
                    log::debug!(
                        "updated sverl local values in {:.3}s",
                        timer.tick().as_secs_f64()
                    );
                }
            }
        }
    }

    fn ai_move(&mut self) {
        let Some(player) = self.model.current_player() else {
            return;
        };

        let action = match self.policy {
            Policy::Random => {
                let action = random_action(&self.model);
                log::debug!("random chose action {:?}", action);
                action
            }
            Policy::Minimax => {
                let (action, value) =
                    minimax_action(&self.model, &mut self.minimax_cache, player, None);
                log::debug!("minimax chose action {:?} with value {:.2}", action, value);
                action
            }
        };
        self.model.set(action, player.into());
        self.update_values(true);
    }

    fn human_move(&mut self, pos: vec2<Coord>) {
        if !self.model.check(pos) {
            return;
        }
        let Some(player) = self.model.current_player() else {
            return;
        };
        self.model.set(pos, player.into());
        self.update_values(true);
    }

    fn reset(&mut self) {
        self.model = Grid::new();
        self.update_values(true);
    }

    fn click(&mut self, pos: vec2<f32>, button: geng::MouseButton) {
        let geng::MouseButton::Left = button else {
            return;
        };

        if self.ui.policy_random.contains(pos) {
            self.policy = Policy::Random;
            self.update_values(true);
        } else if self.ui.policy_minimax.contains(pos) {
            self.policy = Policy::Minimax;
            self.update_values(true);
        } else if self.ui.method_shapley.contains(pos) {
            self.method = Method::Shapley;
            self.update_values(false);
        } else if self.ui.method_sverl.contains(pos) {
            self.method = Method::Sverl {
                global: self.sverl_global,
            };
            self.update_values(false);
        } else if self.ui.board_reset.contains(pos) {
            self.reset();
        } else if self.ui.board_policy_turn.contains(pos) {
            self.ai_move();
        } else if self.ui.method_sverl_global.contains(pos) {
            if let Method::Sverl { global } = &mut self.method {
                *global = !*global;
                self.sverl_global = *global;
                self.update_values(false);
            }
        } else {
            let world_pos = self
                .camera
                .screen_to_world(self.framebuffer_size.as_f32(), pos.as_f32());
            let cell_pos = world_pos.map(|x| x.floor() as isize);
            if cell_pos.x >= 0 && cell_pos.y >= 0 {
                let cell_pos = cell_pos.map(|x| x as Coord);
                self.human_move(cell_pos);
            }
        }
    }

    fn draw_ui(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let camera = &geng::PixelPerfectCamera;
        let font_size = self.ui.font_size;

        let mut draw_button = |text: &str, position: Aabb2<f32>, active: bool| {
            let color = if active {
                self.config.palette.button_border_active
            } else {
                self.config.palette.button_border
            };
            self.geng
                .draw2d()
                .quad(framebuffer, camera, position, color);

            let color = if position.contains(self.ui.cursor_pos) {
                self.config.palette.button_background_hover
            } else {
                self.config.palette.button_background
            };
            self.geng.draw2d().quad(
                framebuffer,
                camera,
                position.extend_uniform(-font_size * 0.2),
                color,
            );

            self.geng.default_font().draw(
                framebuffer,
                camera,
                text,
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(position.center())
                    * mat3::scale_uniform(font_size)
                    * mat3::translate(vec2(0.0, 0.25)),
                self.config.palette.text,
            );
        };

        draw_button(
            "Policy: Random",
            self.ui.policy_random,
            matches!(self.policy, Policy::Random),
        );
        draw_button(
            "Policy: Minimax",
            self.ui.policy_minimax,
            matches!(self.policy, Policy::Minimax),
        );
        draw_button(
            "Method: Shapley",
            self.ui.method_shapley,
            matches!(self.method, Method::Shapley),
        );
        draw_button(
            "Method: SVERL-P",
            self.ui.method_sverl,
            matches!(self.method, Method::Sverl { .. }),
        );
        draw_button("Board Reset", self.ui.board_reset, false);
        draw_button("Policy Turn", self.ui.board_policy_turn, false);

        if let Method::Sverl { global } = self.method {
            // Tickbox
            let position = self.ui.method_sverl_global;

            // Outline
            let color = if global {
                self.config.palette.button_border_active
            } else {
                self.config.palette.button_border
            };
            self.geng
                .draw2d()
                .draw2d(framebuffer, camera, &draw2d::Quad::new(position, color));

            // Fill
            let color = if position.contains(self.ui.cursor_pos) {
                self.config.palette.button_background_hover
            } else {
                self.config.palette.button_background
            };
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(position.extend_uniform(-font_size * 0.2), color),
            );

            // Text
            let font_size = font_size * 0.8;
            self.geng.default_font().draw(
                framebuffer,
                camera,
                "Global",
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(position.center() + vec2(0.0, -font_size / 4.0))
                    * mat3::scale_uniform(font_size),
                Rgba::WHITE,
            );
        }
    }
}

impl geng::State for State {
    fn update(&mut self, _delta_time: f64) {
        self.camera.center = self.model.bounds().map(|x| x as f32).center();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::Space => {
                    self.ai_move();
                }
                geng::Key::R => {
                    self.reset();
                }
                _ => {}
            },
            geng::Event::MousePress { button } => {
                if let Some(mouse_pos) = self.geng.window().cursor_position() {
                    self.click(mouse_pos.as_f32(), button);
                }
            }
            geng::Event::CursorMove { position } => {
                self.ui.cursor_pos = position.as_f32();
            }
            _ => (),
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.ui.layout(framebuffer.size().as_f32());

        self.framebuffer_size = framebuffer.size();
        ugli::clear(
            framebuffer,
            Some(self.config.palette.background),
            None,
            None,
        );

        // Grid lines
        let bounds = self.model.bounds();
        for x in 1..bounds.width() {
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Segment::new(
                    Segment(
                        vec2(x, bounds.min.y).as_f32(),
                        vec2(x, bounds.max.y).as_f32(),
                    ),
                    0.1,
                    self.config.palette.grid,
                ),
            );
        }
        for y in 1..bounds.height() {
            self.geng.draw2d().draw2d(
                framebuffer,
                &self.camera,
                &draw2d::Segment::new(
                    Segment(
                        vec2(bounds.min.x, y).as_f32(),
                        vec2(bounds.max.x, y).as_f32(),
                    ),
                    0.1,
                    self.config.palette.grid,
                ),
            );
        }

        // Cells
        // let minimax = self.model.current_player().map_or(Grid::zero(), |player| {
        //     minimax(&self.model, &mut self.minimax_cache, player, None, 0)
        // });
        for pos in self.model.positions() {
            if let Some(cell) = self.model.get(pos) {
                let value = match self.method {
                    Method::Shapley => self
                        .geng
                        .window()
                        .cursor_position()
                        .and_then(|mouse_pos| {
                            let mouse_pos = self.camera.screen_to_world(
                                self.framebuffer_size.as_f32(),
                                mouse_pos.as_f32(),
                            );
                            let cell_pos = mouse_pos.map(|x| x.floor() as isize);
                            (cell_pos.x >= 0 && cell_pos.y >= 0)
                                .then(|| {
                                    let cell_pos = cell_pos.map(|x| x as Coord);
                                    self.shapley_values
                                        .as_ref()
                                        .and_then(|values| values.get(cell_pos))
                                        .and_then(|grid| grid.get(pos))
                                        .copied()
                                })
                                .flatten()
                        })
                        .unwrap_or(0.0) as f32,
                    Method::Sverl { global } => {
                        let values = if global {
                            &self.sverl_values_global
                        } else {
                            &self.sverl_values_local
                        };
                        values
                            .as_ref()
                            .and_then(|values| values.get(pos))
                            .copied()
                            .unwrap_or(0.0) as f32
                    }
                };

                let ratio = 0.9;
                let aabb = Aabb2::point(pos.as_f32() + vec2(0.5, 0.5))
                    .extend_symmetric(vec2(ratio, ratio) / 2.0);
                let mut negative = self.config.palette.eval_negative;
                negative.a = -value;
                let mut positive = self.config.palette.eval_positive;
                positive.a = value;
                let color = if value > 0.0 { positive } else { negative };
                self.geng
                    .draw2d()
                    .quad(framebuffer, &self.camera, aabb, color);

                match cell {
                    Tile::Empty => {}
                    Tile::X => {
                        self.draw_x(pos, 1.0, framebuffer);
                    }
                    Tile::O => {
                        self.draw_o(pos, 1.0, framebuffer);
                    }
                }

                // if self.model.check(pos) {
                //     let value = minimax.get(pos).unwrap();
                //     self.geng.default_font().draw(
                //         framebuffer,
                //         &self.camera,
                //         &format!("{:+.2}", value),
                //         vec2::splat(geng::TextAlign::CENTER),
                //         mat3::translate(pos.as_f32() + vec2(0.5, 0.5)) * mat3::scale_uniform(0.3),
                //         self.config.palette.text,
                //     );
                // }
            }
        }

        if let Some(winner) = self.model.winner() {
            self.geng.default_font().draw(
                framebuffer,
                &self.camera,
                &format!("Winner {:?}", winner),
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(self.camera.center + vec2(0.0, -4.0)) * mat3::scale_uniform(1.0),
                self.config.palette.text,
            );
        } else {
            // Display minimax evaluation
            if let Some(mouse_pos) = self.geng.window().cursor_position() {
                let mouse_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size.as_f32(), mouse_pos.as_f32());
                let cell_pos = mouse_pos.map(|x| x.floor() as isize);
                if cell_pos.x >= 0 && cell_pos.y >= 0 {
                    let cell_pos = cell_pos.map(|x| x as Coord);
                    if let Some(Tile::Empty) = self.model.get(cell_pos) {
                        let mut grid = self.model.clone();
                        if let Some(player) = grid.current_player() {
                            grid.set(cell_pos, player.into());
                            let (_action, value) =
                                minimax_action(&grid, &mut self.minimax_cache, player.next(), None);

                            match player {
                                Player::X => self.draw_x(cell_pos, 0.5, framebuffer),
                                Player::O => self.draw_o(cell_pos, 0.5, framebuffer),
                            }

                            self.geng.default_font().draw(
                                framebuffer,
                                &self.camera,
                                &format!("Minimax evaluation: {:+.2}", value),
                                vec2::splat(geng::TextAlign::CENTER),
                                mat3::translate(self.camera.center + vec2(0.0, -4.0))
                                    * mat3::scale_uniform(0.8),
                                self.config.palette.text,
                            );
                        }
                    }
                }
            }
        }

        self.draw_ui(framebuffer);
    }
}
