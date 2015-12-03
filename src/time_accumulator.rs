pub struct TimeAccumulator {
    _accumulator: f64,
    _step: f64
}

impl TimeAccumulator {
    pub fn new(step: f64) -> TimeAccumulator {
        TimeAccumulator {
            _accumulator: 0.0f64,
            _step: step
        }
    }

    pub fn update(&mut self,
                  delta: f64) -> &mut TimeAccumulator {
        self._accumulator += delta;
        self
    }
}

impl Iterator for TimeAccumulator {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if self._accumulator >= self._step {
            self._accumulator -= self._step;
            Some(self._step)
        } else {
            None
        }
    }
}
