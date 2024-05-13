use crate::tictactoe::{Tile, *};

use std::collections::BTreeMap;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct State {
    geng: Geng,
    framebuffer_size: vec2<usize>,
    camera: Camera2d,
    model: Grid,
    minimax_cache: BTreeMap<Grid, Grid<f64>>,
    shapley_values: Grid<Grid<f64>>,
    sverl_values: Grid<f64>,
}

impl State {
    pub fn new(geng: &Geng) -> State {
        let mut state = State {
            geng: geng.clone(),
            framebuffer_size: vec2(1, 1),
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            model: Grid::new(),
            minimax_cache: BTreeMap::new(),
            shapley_values: Grid::from_fn(|_| Grid::zero()),
            sverl_values: Grid::zero(),
        };
        state.update_values();
        state
    }

    fn draw_x(&self, pos: vec2<usize>, transparency: f32, framebuffer: &mut ugli::Framebuffer) {
        let ratio = 0.7;
        let aabb =
            Aabb2::point(pos.as_f32() + vec2(0.5, 0.5)).extend_symmetric(vec2(ratio, ratio) / 2.0);

        let mut color = Rgba::GRAY;
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

        let mut color = Rgba::GRAY;
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

    fn update_values(&mut self) {
        let mut policy = policy_minimax_cached(None, &mut self.minimax_cache);
        self.shapley_values = self.model.shapley(&mut policy);
        self.sverl_values = self.model.sverl_local(0.9, &mut policy);
    }
}

impl geng::State for State {
    fn update(&mut self, _delta_time: f64) {
        self.camera.center = self.model.bounds().map(|x| x as f32).center();
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress { key } => {
                match key {
                    geng::Key::Space => {
                        // AI move
                        if let Some(player) = self.model.current_player() {
                            let (action, value) =
                                minimax_action(&self.model, &mut self.minimax_cache, player, None);
                            log::debug!(
                                "minimax chose action {:?} with value {:.2}",
                                action,
                                value
                            );
                            self.model.set(action, player.into());
                            self.update_values();
                        }
                    }
                    geng::Key::R => {
                        self.model = Grid::new();
                        self.update_values();
                    }
                    _ => {}
                }
            }
            geng::Event::MousePress { button } => {
                if let Some(mouse_pos) = self.geng.window().cursor_position() {
                    let mouse_pos = self
                        .camera
                        .screen_to_world(self.framebuffer_size.as_f32(), mouse_pos.as_f32());
                    let cell_pos = mouse_pos.map(|x| x.floor() as Coord);
                    match button {
                        geng::MouseButton::Left => {
                            self.model.set(cell_pos, Tile::X);
                        }
                        geng::MouseButton::Right => {
                            self.model.set(cell_pos, Tile::O);
                        }
                        geng::MouseButton::Middle => {
                            self.model.set(cell_pos, Tile::Empty);
                        }
                    }
                    self.update_values();
                }
            }
            _ => (),
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

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
                    Rgba::GRAY,
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
                    Rgba::GRAY,
                ),
            );
        }

        // Cells
        // let minimax = self.model.current_player().map_or(Grid::zero(), |player| {
        //     minimax(&self.model, &mut self.minimax_cache, player, None, 0)
        // });
        for pos in self.model.positions() {
            if let Some(cell) = self.model.get(pos) {
                // let value = self
                //     .geng
                //     .window()
                //     .cursor_position()
                //     .and_then(|mouse_pos| {
                //         let mouse_pos = self
                //             .camera
                //             .screen_to_world(self.framebuffer_size.as_f32(), mouse_pos.as_f32());
                //         let cell_pos = mouse_pos.map(|x| x.floor() as Coord);
                //         self.shapley_values
                //             .get(cell_pos)
                //             .and_then(|grid| grid.get(pos))
                //             .copied()
                //     })
                //     .unwrap_or(0.0) as f32;
                let value = self.sverl_values.get(pos).copied().unwrap_or(0.0) as f32;

                let ratio = 0.9;
                let aabb = Aabb2::point(pos.as_f32() + vec2(0.5, 0.5))
                    .extend_symmetric(vec2(ratio, ratio) / 2.0);
                let mut negative = Rgba::RED;
                negative.a = -value;
                let mut positive = Rgba::BLUE;
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
                //         Rgba::WHITE,
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
                Rgba::WHITE,
            );
        } else {
            // Display minimax evaluation
            if let Some(mouse_pos) = self.geng.window().cursor_position() {
                let mouse_pos = self
                    .camera
                    .screen_to_world(self.framebuffer_size.as_f32(), mouse_pos.as_f32());
                let cell_pos = mouse_pos.map(|x| x.floor() as Coord);
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
                        // match player.next() {
                        //     Player::X => self.draw_x(action, 0.25, framebuffer),
                        //     Player::O => self.draw_o(action, 0.25, framebuffer),
                        // }

                        self.geng.default_font().draw(
                            framebuffer,
                            &self.camera,
                            &format!("Minimax evaluation: {:+.2}", value),
                            vec2::splat(geng::TextAlign::CENTER),
                            mat3::translate(self.camera.center + vec2(0.0, -4.0))
                                * mat3::scale_uniform(0.8),
                            Rgba::WHITE,
                        );
                        // self.geng.default_font().draw(
                        //     framebuffer,
                        //     &self.camera,
                        //     &format!(
                        //         "Shapley value: {:.2}",
                        //         self.values.get(cell_pos).copied().unwrap_or(0.0)
                        //     ),
                        //     vec2::splat(geng::TextAlign::CENTER),
                        //     mat3::translate(self.camera.center + vec2(0.0, -3.0))
                        //         * mat3::scale_uniform(0.8),
                        //     Rgba::WHITE,
                        // );
                    }
                }
            }
        }
    }
}
