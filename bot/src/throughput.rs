pub struct RollingRecorder<const SIZE: usize> {
    buffer: [f64; SIZE],
    index: usize,
}

impl<const SIZE: usize> RollingRecorder<SIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [0.0; SIZE],
            index: 0,
        }
    }

    pub fn record(&mut self, value: f64) {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % SIZE;
    }

    pub fn get_average(&self) -> f64 {
        self.buffer.iter().sum::<f64>() / (SIZE as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_recorder() {
        let mut recorder = RollingRecorder::<5>::new();
        for i in 1..=5 {
            assert_eq!(recorder.index, i - 1);
            recorder.record(i as f64);
        }
        assert_eq!(recorder.index, 0);
        assert_eq!(recorder.get_average(), 3.0);

        recorder.record(6.0);
        assert_eq!(recorder.get_average(), 4.0);
    }
}
