use board::Board;
use rand::{Rng, StdRng};
use std::cmp::{min, max};

pub trait Simulation<T> {
    fn advance(&mut self);
}

pub struct TorusNeighbors {
    x: usize,
    y: usize,
    idx: usize,
    x_end: usize,
    y_end: usize
}

fn torus_sub_1(idx: usize,
               idx_end: usize) -> usize {
    if idx == 0 {
        idx_end - 1
    } else if idx > idx_end {
        assert!(idx == idx_end + 1);
        0
    } else {
        idx - 1
    }
}

impl Iterator for TorusNeighbors {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.idx == 9 {
            None
        } else {
            let x = torus_sub_1(self.x + (self.idx % 3), self.x_end);
            let y = torus_sub_1(self.y + (self.idx / 3), self.y_end);

            self.idx += 1;
            if self.idx == 4 {
                self.idx += 1;
            }

            Some((x, y))
        }
    }
}

pub fn torus_neighbors(x: usize,
                       y: usize,
                       x_end: usize,
                       y_end: usize) -> TorusNeighbors {
    TorusNeighbors {
        x: x,
        y: y,
        idx: 0,
        x_end: x_end,
        y_end: y_end
    }
}

#[test]
fn test_torus_neighbors_basic() {
    let expected_output = [
        (0, 0), (1, 0), (2, 0),
        (0, 1),         (2, 1),
        (0, 2), (1, 2), (2, 2)
    ];

    assert_point_iterables_eq(&expected_output,
                              &mut torus_neighbors(1, 1, 3, 3));
}

#[test]
fn test_torus_neighbors_zero() {
    let expected_output = [
        (2, 2), (0, 2), (1, 2),
        (2, 0),         (1, 0),
        (2, 1), (0, 1), (1, 1)
    ];

    assert_point_iterables_eq(&expected_output,
                              &mut torus_neighbors(0, 0, 3, 3));
}

#[test]
fn test_torus_neighbors_end() {
    let expected_output = [
        (1, 1), (2, 1), (0, 1),
        (1, 2),         (0, 2),
        (1, 0), (2, 0), (0, 0)
    ];

    assert_point_iterables_eq(&expected_output,
                              &mut torus_neighbors(2, 2, 3, 3));
}
struct GameOfLife {
    board: Board<bool>
}

impl GameOfLife {
    fn count_alive_neighbors(board: &Board<bool>,
                             x: usize,
                             y: usize) -> usize {
        let mut nbrs_alive = 0;

        for (nbr_x, nbr_y) in torus_neighbors(x, y, board.width, board.height) {
            if board.at(nbr_x, nbr_y) {
                nbrs_alive += 1;
            }
        }

        nbrs_alive
    }

    fn advance_board(old: &Board<bool>) -> Board<bool> {
        let mut new = Board::new(old.width, old.height, false);

        for (x, y) in old.indices() {
            let is_alive = old.at(x, y);
            let nbrs_alive = GameOfLife::count_alive_neighbors(old, x, y);

            *new.at_mut(x, y) = (!is_alive && nbrs_alive == 3)
                             || (is_alive && (nbrs_alive == 2 || nbrs_alive == 3));
        }

        new
    }
}

impl Simulation<bool> for GameOfLife {
    fn advance(&mut self) {
        self.board = GameOfLife::advance_board(&self.board);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Field {
    Empty,
    Occupied { energy: f32 }
}

pub struct GoodEvilConfig {
    pub num_specimens: usize,
    pub initial_specimen_energy: f32,
    pub energy_loss_per_step: f32,
    pub deadly_energy_margin: f32
}

pub struct GoodEvil {
    pub cfg: GoodEvilConfig,
    rng: Box<StdRng>,
    collision_energy: f32,
    pub board: Board<Field>
}

impl GoodEvil {
    fn find_empty_field(board: &Board<Field>,
                        rng: &mut StdRng) -> (usize, usize) {
        let loop_limit = board.width * board.height;
        let mut coords = None;

        for _ in 0..loop_limit {
            let x = rng.gen_range(0, board.width);
            let y = rng.gen_range(0, board.height);

            if board.at(x, y) == Field::Empty {
                coords = Some((x, y));
                break
            }
        }

        if let Some(xy) = coords {
            xy
        } else {
            panic!("could not find an empty field in {} iterations", loop_limit);
        }
    }

    pub fn new(width: usize,
               height: usize,
               cfg: GoodEvilConfig,
               mut rng: Box<StdRng>) -> GoodEvil {
        assert!(cfg.num_specimens <= width * height);

        let mut board = Board::new(width, height, Field::Empty);

        for _ in 0..cfg.num_specimens {
            let (x, y) = GoodEvil::find_empty_field(&board, &mut rng);
            *board.at_mut(x, y) = Field::Occupied { energy: cfg.initial_specimen_energy };
        }

        GoodEvil {
            cfg: cfg,
            rng: rng,
            collision_energy: 0.0f32,
            board: board
        }
    }

    fn handle_collision(source: Field,
                        target: Field) -> (Field, Field) {
        match (source, target) {
            (Field::Occupied { energy: src_energy },
             Field::Occupied { energy: tgt_energy }) => {
                let total_energy = src_energy + tgt_energy;
                let half = total_energy * 0.5f32;

                (Field::Occupied { energy: half },
                 Field::Occupied { energy: half })
            }
            _ => panic!("should never happen")
        }
    }

    fn get_new_coords(x: usize,
                      y: usize,
                      board: &Board<Field>,
                      rng: &mut StdRng) -> (usize, usize) {
        let min_x = max(0i64, x as i64 - 1) as usize;
        let max_x = min(x + 2, board.width);

        let min_y = max(0i64, y as i64 - 1) as usize;
        let max_y = min(y + 2, board.height);

        (rng.gen_range(min_x, max_x),
         rng.gen_range(min_y, max_y))
    }

    fn move_specimen(&mut self,
                     src_x: usize,
                     src_y: usize,
                     src_energy: f32,
                     dst_x: usize,
                     dst_y: usize,
                     new: &mut Board<Field>) {
        assert!(src_x != dst_x || src_y != dst_y);

        // make sure we may skip the move in case of a collision
        assert!(new.at(src_x, src_y) == Field::Empty);

        match new.at(dst_x, dst_y) {
            Field::Empty => {
                *new.at_mut(dst_x, dst_y) = Field::Occupied { energy: src_energy }
            },
            dst_field => {
                let src = Field::Occupied { energy: src_energy };
                let (new_src, new_tgt) = GoodEvil::handle_collision(src, dst_field);

                *new.at_mut(src_x, src_y) = new_src;
                *new.at_mut(dst_x, dst_y) = new_tgt;
            }
        }
    }

    fn update_specimen(&mut self,
                       x: usize,
                       y: usize,
                       new: &mut Board<Field>) {
        match self.board.at(x, y) {
            Field::Empty => (),
            Field::Occupied { energy } => {
                let new_energy = energy - self.cfg.energy_loss_per_step;

                let (target_x, target_y) = GoodEvil::get_new_coords(x, y, &new, &mut self.rng);

                if x == target_x && y == target_y {
                    *new.at_mut(x, y) = Field::Occupied { energy: new_energy }
                } else if self.board.at(target_x, target_y) != Field::Empty {
                    // the target field is possibly taken by a specimen that may not be able to
                    // move; stay in place in that case to avoid further problems
                    *new.at_mut(x, y) = Field::Occupied { energy: new_energy }
                } else {
                    self.move_specimen(x, y, new_energy, target_x, target_y, new);
                }

            }
        }
    }
}

impl Simulation<Field> for GoodEvil {
    fn advance(&mut self) {
        let mut new = Board::new(self.board.width, self.board.height, Field::Empty);

        for (x, y) in self.board.indices() {
            self.update_specimen(x, y, &mut new);
        }

        self.board = new;
    }
}
