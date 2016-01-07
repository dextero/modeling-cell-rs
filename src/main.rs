extern crate rand;
extern crate getopts;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use std::fmt;

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

struct Options {
    board_size: (usize, usize)
}

enum ParseResult {
    Success(Options),
    Failure(String),
    Exit
}

impl Options {
    fn parse_csv_ints(string: &String) -> Result<Vec<usize>, String> {
        let mut sizes = Vec::new();
        for substr in string.split(",") {
            sizes.push(match substr.parse::<usize>() {
                Ok(v) => v,
                Err(e) => return Err(e.to_string())
            })
        }

        Ok(sizes)
    }

    fn parse_size(string: Option<String>) -> Result<(usize,usize), String> {
        match string {
            None => Ok((80, 60)),
            Some(s) => {
                let sizes = try!(Options::parse_csv_ints(&s));

                match sizes.len() {
                    1 => Ok((sizes[0], sizes[0])),
                    2 => Ok((sizes[0], sizes[1])),
                    _ => {
                        let msg = format!("invalid argument format: {}, expected SIZE or WIDTH,HEIGHT",
                                          s);
                        Err(msg)
                    }
                }
            }
        }
    }

    pub fn from_cmdline() -> ParseResult {
        let args: Vec<String> = std::env::args().collect();

        let mut opts = getopts::Options::new();
        opts.optopt("s", "board-size", "set board size", "WIDTH,HEIGHT");
        opts.optflag("h", "help", "print this help message");

        let matches = match opts.parse(&args[1..]) {
            Ok(matches) => matches,
            Err(e) => return ParseResult::Failure(e.to_string())
        };

        if matches.opt_present("h") {
            let msg = format!("Usage: {} [options]", args[0]);
            print!("{}", opts.usage(&msg));
            return ParseResult::Exit
        }

        let board_size = match Options::parse_size(matches.opt_str("s")) {
            Ok(size) => size,
            Err(e) => return ParseResult::Failure(e)
        };

        ParseResult::Success(Options {
                board_size: board_size
        })
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "board_size: {}, {}", self.board_size.0, self.board_size.1)
    }
}

fn main() {
    let opts = match Options::from_cmdline() {
        ParseResult::Success(opts) => opts,
        ParseResult::Failure(reason) => {
            println!("{}", reason);
            return
        },
        ParseResult::Exit => return
    };

    println!("Configuration:\n{}", opts);

    let gl_version = OpenGL::V3_2;

    let window: Window = WindowSettings::new("cell", [800, 600])
            .opengl(gl_version)
            .exit_on_esc(true)
            .build()
            .unwrap();

    let mut app = App {
        gl: GlGraphics::new(gl_version),
        board: Board::new_random(opts.board_size.0,
                                 opts.board_size.1),
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
