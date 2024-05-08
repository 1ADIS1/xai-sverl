use crate::tictactoe::{Cell, *};

use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

pub struct State {
    geng: Geng,
    framebuffer_size: vec2<usize>,
    camera: Camera2d,
    model: Grid,
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
        }
    }
}

impl geng::State for State {
    fn update(&mut self, _delta_time: f64) {
        self.camera.center = self.model.bounds().map(|x| x as f32).center();
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::MousePress { button } = event {
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
                let ratio = 0.8;
                let aabb = Aabb2::point(pos.as_f32() + vec2(0.5, 0.5))
                    .extend_symmetric(vec2(ratio, ratio) / 2.0);
                match cell {
                    Cell::Empty => continue,
                    Cell::X => {
                        self.geng.draw2d().draw2d(
                            framebuffer,
                            &self.camera,
                            &draw2d::Segment::new(
                                Segment(aabb.bottom_left(), aabb.top_right()),
                                0.1,
                                Rgba::RED,
                            ),
                        );
                        self.geng.draw2d().draw2d(
                            framebuffer,
                            &self.camera,
                            &draw2d::Segment::new(
                                Segment(aabb.top_left(), aabb.bottom_right()),
                                0.1,
                                Rgba::RED,
                            ),
                        );
                    }
                    Cell::O => {
                        self.geng.draw2d().draw2d(
                            framebuffer,
                            &self.camera,
                            &draw2d::Ellipse::circle_with_cut(
                                aabb.center(),
                                aabb.width() / 2.0 * ratio * 0.9,
                                aabb.width() / 2.0 * ratio,
                                Rgba::CYAN,
                            ),
                        );
                    }
                }
            }
        }
    }
}
