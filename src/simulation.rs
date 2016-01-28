use board::Board;
use rand::{Rng, StdRng};
use std::cmp::{min, max};
use std::iter::Iterator;
use std::collections::HashMap;
use std::mem;

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
            if *board.at(nbr_x, nbr_y) {
                nbrs_alive += 1;
            }
        }

        nbrs_alive
    }

    fn advance_board(old: &Board<bool>) -> Board<bool> {
        let mut new = Board::new(old.width, old.height, false);

        for (x, y) in old.indices() {
            let is_alive = *old.at(x, y);
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
pub struct Specimen {
    pub energy: f32
}

#[derive(Clone, PartialEq)]
pub enum Field {
    Empty,
    Occupied(Specimen),
    Collision(Vec<Specimen>)
}

pub struct GoodEvilConfig {
    pub num_specimens: usize,
    pub initial_specimen_energy: f32,
    pub energy_loss_per_step: f32,
    pub deadly_energy_margin: f32
}

type CollisionMap = HashMap<(usize, usize), Vec<Specimen>>;

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

            if *board.at(x, y) == Field::Empty {
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
        assert!(width >= 2);
        assert!(height >= 2);
        assert!(cfg.num_specimens <= width * height);

        let mut board = Board::new(width, height, Field::Empty);

        for _ in 0..cfg.num_specimens {
            let (x, y) = GoodEvil::find_empty_field(&board, &mut rng);
            *board.at_mut(x, y) = Field::Occupied(Specimen { energy: cfg.initial_specimen_energy });
        }

        GoodEvil {
            cfg: cfg,
            rng: rng,
            collision_energy: 0.0f32,
            board: board
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

    fn move_specimen(specimen: Specimen,
                     dst_x: usize,
                     dst_y: usize,
                     new: &mut Board<Field>) {
        let target_field: &mut Field = new.at_mut(dst_x, dst_y);

        match target_field {
            &mut Field::Empty => {
                *target_field = Field::Occupied(specimen);
            },
            &mut Field::Occupied(tgt_specimen) => {
                let specimens = vec!(tgt_specimen, specimen);
                //println!("collision, create");
                *target_field = Field::Collision(specimens.clone());
            },
            &mut Field::Collision(ref mut specimens) => {
                specimens.push(specimen);
            }
        }
    }

    fn update_specimen(&mut self,
                       x: usize,
                       y: usize,
                       new: &mut Board<Field>) {
        match self.board.at(x, y) {
            &Field::Empty => (),
            &Field::Occupied(specimen) => {
                let new_energy = specimen.energy - self.cfg.energy_loss_per_step;
                let (target_x, target_y) = GoodEvil::get_new_coords(x, y, &new, &mut self.rng);

                GoodEvil::move_specimen(specimen, target_x, target_y, new);
            },
            &Field::Collision(_) => panic!("should never happen")
        }
    }

    fn surrounding_fields(x: usize,
                          y: usize,
                          board: &Board<Field>) -> Vec<(usize, usize)> {
        let mut fields = vec!();

        let min_x = max(0i64, x as i64 - 1) as usize;
        let max_x = min(x + 2, board.width);

        let min_y = max(0i64, y as i64 - 1) as usize;
        let max_y = min(y + 2, board.height);

        for x in min_x..max_x {
            for y in min_y..max_y {
                fields.push((x, y))
            }
        }

        fields
    }

    fn assign_neighbors(x: usize,
                        y: usize,
                        num_elems: usize,
                        board: &Board<Field>,
                        rng: &mut StdRng) -> Vec<(usize, usize)> {
        let mut fields = GoodEvil::surrounding_fields(x, y, board);
        assert!(num_elems <= fields.len());

        rng.shuffle(&mut fields[..]); 
        fields.resize(num_elems, (-1i32 as usize, -1i32 as usize));
        fields
    }

    fn resolve_collisions(rng: &mut StdRng,
                          old: &Board<Field>) -> Board<Field> {
        let mut new = Board::new(old.width, old.height, Field::Empty);

        for (x, y) in old.indices() {
            match old.at(x, y) {
                &Field::Empty => (),
                &Field::Occupied(ref specimen) => {
                    GoodEvil::move_specimen(specimen.clone(), x, y, &mut new);
                },
                &Field::Collision(ref specimens) => {
                    let positions = GoodEvil::assign_neighbors(x, y, specimens.len(), &new, rng);

                    *new.at_mut(x, y) = Field::Empty;

                    for ((new_x, new_y), specimen) in positions.into_iter().zip(specimens) {
                        GoodEvil::move_specimen(specimen.clone(), new_x, new_y, &mut new);
                    }
                }
            }
        }

        new
    }

    fn has_collisions(board: &Board<Field>) -> bool {
        board.iter().any(|f| match f {
                             &Field::Collision(_) => true,
                             _ => false
                         })
    }

    fn count_specimens(board: &Board<Field>) -> usize {
        board.iter().fold(0, |sum, f| match f {
            &Field::Empty => sum,
            &Field::Occupied(_) => sum + 1,
            &Field::Collision(ref specimens) => sum + specimens.len()
        })
    }
}

impl Simulation<Field> for GoodEvil {
    fn advance(&mut self) {
        println!("advance");

        let mut new = Board::new(self.board.width, self.board.height, Field::Empty);

        for (x, y) in self.board.indices() {
            self.update_specimen(x, y, &mut new);
        }

        self.board = new;

        let mut coll_iters = 0;
        let specimens = GoodEvil::count_specimens(&self.board);
        while GoodEvil::has_collisions(&self.board) {
            coll_iters += 1;
            println!("resolve_collisions, iteration {}, {} specimens",
                     coll_iters, GoodEvil::count_specimens(&self.board));

            assert!(GoodEvil::count_specimens(&self.board) == specimens);
            self.board = GoodEvil::resolve_collisions(&mut self.rng, &self.board);
        }

        //GoodEvil::debug_collisions(&self.board, &self.collisions);
    }
}
