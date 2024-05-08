use crate::tictactoe::{Cell, *};

use std::collections::BTreeMap;

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct State {
    geng: Geng,
    framebuffer_size: vec2<usize>,
    camera: Camera2d,
    model: Grid,
    minimax_cache: BTreeMap<Grid, (Action, f64)>,
}

impl State {
    pub fn new(geng: &Geng) -> State {
        State {
            geng: geng.clone(),
            framebuffer_size: vec2(1, 1),
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            model: Grid::new(),
            minimax_cache: BTreeMap::new(),
        }
    }

    fn draw_x(&self, pos: vec2<usize>, transparency: f32, framebuffer: &mut ugli::Framebuffer) {
        let ratio = 0.8;
        let aabb =
            Aabb2::point(pos.as_f32() + vec2(0.5, 0.5)).extend_symmetric(vec2(ratio, ratio) / 2.0);

        let mut color = Rgba::RED;
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
        let ratio = 0.8;
        let aabb =
            Aabb2::point(pos.as_f32() + vec2(0.5, 0.5)).extend_symmetric(vec2(ratio, ratio) / 2.0);

        let mut color = Rgba::CYAN;
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
                                minimax(&self.model, &mut self.minimax_cache, player, None, 0);
                            log::debug!(
                                "minimax chose action {:?} with value {:.2}",
                                action,
                                value
                            );
                            self.model.set(action, player.into());
                        }
                    }
                    geng::Key::R => {
                        self.model = Grid::new();
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
                            self.model.set(cell_pos, Cell::X);
                        }
                        geng::MouseButton::Right => {
                            self.model.set(cell_pos, Cell::O);
                        }
                        geng::MouseButton::Middle => {
                            self.model.set(cell_pos, Cell::Empty);
                        }
                    }
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
        for pos in self.model.positions() {
            if let Some(cell) = self.model.get(pos) {
                match cell {
                    Cell::Empty => continue,
                    Cell::X => {
                        self.draw_x(pos, 1.0, framebuffer);
                    }
                    Cell::O => {
                        self.draw_o(pos, 1.0, framebuffer);
                    }
                }
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
                if let Some(Cell::Empty) = self.model.get(cell_pos) {
                    let mut grid = self.model.clone();
                    if let Some(player) = grid.current_player() {
                        grid.set(cell_pos, player.into());
                        let (action, value) =
                            minimax(&grid, &mut self.minimax_cache, player.next(), None, 0);

                        match player {
                            Player::X => self.draw_x(cell_pos, 0.5, framebuffer),
                            Player::O => self.draw_o(cell_pos, 0.5, framebuffer),
                        }
                        match player.next() {
                            Player::X => self.draw_x(action, 0.25, framebuffer),
                            Player::O => self.draw_o(action, 0.25, framebuffer),
                        }

                        self.geng.default_font().draw(
                            framebuffer,
                            &self.camera,
                            &format!("Minimax evaluation: {:.2}", -value),
                            vec2::splat(geng::TextAlign::CENTER),
                            mat3::translate(self.camera.center + vec2(0.0, -4.0))
                                * mat3::scale_uniform(1.0),
                            Rgba::WHITE,
                        );
                    }
                }
            }
        }
    }
}
