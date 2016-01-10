use time;

pub struct TickMeter {
    tick_times: [f64; 100],
    idx: usize,
    tick_start_s: f64,
    display_prefix: Option<String>
}

impl TickMeter {
    pub fn new() -> Self {
        TickMeter {
            tick_times: [0.0; 100],
            idx: 0,
            tick_start_s: time::precise_time_s(),
            display_prefix: None
        }
    }

    pub fn with_auto_display(self, display_prefix: &str) -> Self {
        TickMeter {
            display_prefix: Some(display_prefix.to_string()),
            .. self
        }
    }

    pub fn tick(&mut self) {
        let end_s = time::precise_time_s();

        self.tick_times[self.idx] = end_s - self.tick_start_s;
        self.tick_start_s = end_s;
        self.idx = (self.idx + 1) % self.tick_times.len();

        match (&self.display_prefix, self.idx) {
            (&Some(ref prefix), 0) => println!("{}{}", prefix, self.measure()),
            _ => (),
        }
    }

    pub fn measure(&self) -> f64 {
        let total_time_s = self.tick_times.iter().fold(0.0, |sum, x| sum + x);
        self.tick_times.len() as f64 / total_time_s
    }
}
