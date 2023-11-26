
pub use instant::{Instant, Duration};

#[derive(Debug, Clone)]
pub struct StepInterval {
    pub next: Instant,
    pub duration: Duration,
}

impl StepInterval {

    pub fn new(duration: Duration) -> Self {
        Self { next: Instant::now() + duration, duration }
    }

    pub fn from_secs(duration_secs: f64) -> Self {
        Self::new(Duration::from_secs_f64(duration_secs))
    }

    pub fn elapsed(&self) -> i32 {
        let now = Instant::now();
        if now >= self.next {
            ((now - self.next).as_nanos() / self.duration.as_nanos()) as i32 + 1
        } else {
            -(((self.next - now).as_nanos() / self.duration.as_nanos()) as i32)
        }
    }

    pub fn step_by(&mut self, times: i32) {
        if let Some(instant) = {
            if times.is_positive() {
                self.next.checked_add(self.duration * times as u32)
            } else {
                self.next.checked_sub(self.duration * (-times) as u32)
            }
        } {
            self.next = instant
        }
    }

    pub fn step_if_elapsed(&mut self) -> i32 {
        let elapsed = self.elapsed();
        if elapsed >= 1 { self.step_by(elapsed) }
        elapsed
    }

    pub fn step(&mut self) -> i32 {
        let elapsed = self.elapsed();
        // step even if not yet fully elapsed
        if elapsed >= 0 { self.step_by(elapsed.max(1)) }
        elapsed
    }
}



#[derive(Debug, Clone)]
pub struct IntervalCounter {
    pub count: usize,
    pub interval: StepInterval,
}

#[derive(Debug, Clone, Copy)]
pub struct IntervalCount {
    pub count: usize,
    pub times_per_sec: f64,
}

impl IntervalCounter {

    pub fn new(duration: Duration) -> Self {
        Self { count: 0, interval: StepInterval::new(duration) }
    }

    pub fn from_secs(duration_secs: f64) -> Self {
        Self::new(Duration::from_secs_f64(duration_secs))
    }

    pub fn add(&mut self) {
        self.count += 1;
    }

    pub fn count(&mut self) -> Option<IntervalCount> {
        if self.interval.step_if_elapsed() >= 1 {

            let counted = IntervalCount {
                times_per_sec: self.count as f64 / self.interval.duration.as_secs_f64() ,
                count: self.count,
            };

            self.count = 0;

            Some(counted)
        }
        else { None }
    }
}