extern crate rand;
extern crate getopts;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate time;

use std::fmt;
use rand::StdRng;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};

mod board;
mod time_accumulator;
mod tick_meter;
mod simulation;

use time_accumulator::TimeAccumulator;
use tick_meter::TickMeter;
use simulation::{Simulation, Field, GoodEvil, GoodEvilConfig};

struct App {
    gl: GlGraphics,
    simulation: GoodEvil,
    time_accumulator: TimeAccumulator
}

fn color_from_field(field: &Field) -> [f32; 4] {
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    match *field {
        Field::Empty => BLACK,
        Field::Occupied(s) => {
            if s.energy >= 1.0 {
                WHITE
            } else {
                [s.energy, s.energy, s.energy, 1.0]
            }
        },
        Field::Collision(_) => panic!("should never happen")
    }
}

impl App {
    fn render(&mut self,
              args: &RenderArgs) {
        use graphics::*;

        const DARK_BLUE: [f32; 4] = [0.0, 0.0, 0.2, 1.0];

        let board = &self.simulation.board;
        let viewport_rect = args.viewport().rect;
        let elem_size = [viewport_rect[2] as f64 / board.width as f64,
                         viewport_rect[3] as f64 / board.height as f64];

        self.gl.draw(args.viewport(), |ctx, gl| {
            clear(DARK_BLUE, gl);
            println!("draw");

            for (x_idx, y_idx) in board.indices() {
                let color = color_from_field(&board.at(x_idx, y_idx));

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
            self.simulation.advance();
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

    let sim_cfg = GoodEvilConfig {
        num_specimens: opts.board_size.0 * opts.board_size.1 / 20,
        initial_specimen_energy: 1.0f32,
        energy_loss_per_step: 0.01f32,
        deadly_energy_margin: 0.0f32
    };
    let rng = Box::new(StdRng::new().unwrap());

    let mut app = App {
        gl: GlGraphics::new(gl_version),
        simulation: GoodEvil::new(opts.board_size.0,
                                  opts.board_size.1,
                                  sim_cfg, rng),
        time_accumulator: TimeAccumulator::new(0.01f64)
    };

    let mut fps_meter = TickMeter::new().with_auto_display("FPS: ");
    let mut update_meter = TickMeter::new().with_auto_display("Updates/s: ");

    for e in window.events() {
        if let Some(render_args) = e.render_args() {
            app.render(&render_args);
            fps_meter.tick();
        }
        if let Some(update_args) = e.update_args() {
            app.update(&update_args);
            update_meter.tick();
        }
    }
}
