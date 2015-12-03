use std::iter::Iterator;
use rand::{random, Rand};

pub struct Board<T> {
    fields: Box<[T]>,
    width: usize,
    height: usize
}

impl<T: Copy + Rand> Board<T> {
    pub fn new(width: usize,
               height: usize,
               default: T) -> Board<T> {
        Board {
            fields: vec![default; width * height].into_boxed_slice(),
            width: width,
            height: height
        }
    }

    pub fn new_random(width: usize,
                      height: usize) -> Board<T> {
        let mut values = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            values.push(random());
        }

        Board {
            fields: values.into_boxed_slice(),
            width: width,
            height: height
        }
    }

    pub fn at(&self,
              x: usize,
              y: usize) -> T {
        self.fields[y * self.width + x]
    }

    pub fn at_mut(&mut self,
                  x: usize,
                  y: usize) -> &mut T {
        &mut self.fields[y * self.width + x]
    }

    pub fn indices(&self) -> Indices2D {
        indices_2d(self.width, self.height)
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
}

#[test]
fn test_board_at() {
    let mut board = Board::new(4, 3, 0);

    assert_eq!(board.width, 4);
    assert_eq!(board.height, 3);

    for y in 0..board.height {
        for x in 0..board.width {
            let value = y * board.width + x;
            board.fields[value] = value;
            assert_eq!(board.at(x, y), value);
        }
    }
}

pub struct Indices2D {
    x: usize,
    y: usize,
    x_end: usize,
    y_end: usize
}

impl Iterator for Indices2D {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        assert!(self.y <= self.y_end);

        let ret = Some((self.x, self.y));

        if self.y == self.y_end {
            None
        } else {
            self.x += 1;
            if self.x == self.x_end {
                self.x = 0;
                self.y += 1;
            }

            ret
        }
    }
}

pub fn indices_2d(width: usize,
                  height: usize) -> Indices2D {
    Indices2D {
        x: 0,
        y: 0,
        x_end: width,
        y_end: height
    }
}


#[cfg(test)]
fn assert_point_iterables_eq(expected_vals: &[(usize, usize)],
                             actual_it: &mut Iterator<Item=(usize, usize)>) {
    let mut num_compared = 0;
    let mut expected_it = expected_vals.iter();

    while let (Some(actual),
               Some(&expected)) = (actual_it.next(),
                                   expected_it.next()) {
        assert_eq!(expected, actual);
        num_compared += 1;
    }

    assert_eq!(expected_vals.len(), num_compared);
    assert!(actual_it.next().is_none());
}
#[test]
fn test_indices_2d() {
    let expected = [
        (0, 0), (1, 0), (2, 0),
        (0, 1), (1, 1), (2, 1),
        (0, 2), (1, 2), (2, 2),
        (0, 3), (1, 3), (2, 3)
    ];

    assert_point_iterables_eq(&expected, &mut indices_2d(3, 4));
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

pub fn advance(old: &Board<bool>) -> Board<bool> {
    let mut new = Board::new(old.width, old.height, false);

    for (x, y) in indices_2d(old.width, old.height) {
        let is_alive = old.at(x, y);
        let nbrs_alive = count_alive_neighbors(old, x, y);

        *new.at_mut(x, y) = (!is_alive && nbrs_alive == 3)
                         || (is_alive && (nbrs_alive == 2 || nbrs_alive == 3));
    }

    new
}
