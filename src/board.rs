use std::iter::Iterator;
use rand::{random, Rand};

pub struct Board<T> {
    fields: Box<[T]>,
    pub width: usize,
    pub height: usize
}

impl<T> Board<T> {
    pub fn at_mut(&mut self,
                  x: usize,
                  y: usize) -> &mut T {
        &mut self.fields[y * self.width + x]
    }

    pub fn indices(&self) -> Indices2D {
        indices_2d(self.width, self.height)
    }
}

impl<T: Copy> Board<T> {
    pub fn new(width: usize,
               height: usize,
               default: T) -> Board<T> {
        Board {
            fields: vec![default; width * height].into_boxed_slice(),
            width: width,
            height: height
        }
    }

    pub fn at(&self,
              x: usize,
              y: usize) -> T {
        self.fields[y * self.width + x]
    }
}

impl<T: Copy + Rand> Board<T> {
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
