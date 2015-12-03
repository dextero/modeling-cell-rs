extern crate rand;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};

mod board;
mod time_accumulator;

use board::Board;
use time_accumulator::TimeAccumulator;

struct App {
    gl: GlGraphics,
    board: Board<bool>,
    time_accumulator: TimeAccumulator
}

impl App {
    fn render(&mut self,
              args: &RenderArgs) {
        use graphics::*;

        const DARK_BLUE: [f32; 4] = [0.0, 0.0, 0.2, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let viewport_rect = args.viewport().rect;
        let elem_size = [viewport_rect[2] as f64 / self.board.width() as f64,
                         viewport_rect[3] as f64 / self.board.height() as f64];
        let board = &self.board;

        self.gl.draw(args.viewport(), |ctx, gl| {
            clear(DARK_BLUE, gl);

            for (x_idx, y_idx) in board.indices() {
                let color = if board.at(x_idx, y_idx) {
                    WHITE
                } else {
                    BLACK
                };

                let x = x_idx as f64;
                let y = y_idx as f64;

                let rect = [
                    x * elem_size[0],
                    y * elem_size[1],
                    (x + 1.0f64) * elem_size[0],
                    (y + 1.0f64) * elem_size[1],
                ];
                rectangle(color, rect, ctx.transform, gl);
            }
        });
    }

    fn update(&mut self,
              args: &UpdateArgs) {
        for _step in self.time_accumulator.update(args.dt) {
            self.board = board::advance(&self.board);
        };
    }
}

fn main() {
    let gl_version = OpenGL::V3_2;

    let window: Window = WindowSettings::new("cell", [800, 600])
            .opengl(gl_version)
            .exit_on_esc(true)
            .build()
            .unwrap();

    let mut app = App {
        gl: GlGraphics::new(gl_version),
        board: Board::new_random(80, 60),
        time_accumulator: TimeAccumulator::new(0.01f64)
    };

    for e in window.events() {
        if let Some(render_args) = e.render_args() {
            app.render(&render_args);
        }
        if let Some(update_args) = e.update_args() {
            app.update(&update_args);
        }
    }
}
