
pub use instant::{Instant, Duration};


#[derive(Debug, Clone)]
pub struct NormInterval {
  pub instant: Instant,
  pub duration: Duration,
}

impl NormInterval {

    pub fn new(duration: Duration) -> Self {
        Self { instant: Instant::now(), duration }
    }

    pub fn from_secs(duration_secs: f64) -> Self {
        Self::new(Duration::from_secs_f64(duration_secs))
    }

    pub fn elapsed(&self) -> f64 {
        let now = Instant::now();
        if now >= self.instant {
            (now - self.instant).div_duration_f64(self.duration)
        } else {
            -(self.instant - now).div_duration_f64(self.duration)
        }
    }

    pub fn advance_by(&mut self, times: f64) {
        if times.is_sign_positive() {
            self.instant += self.duration.mul_f64(times);
        } else {
            self.instant -= self.duration.mul_f64(-times);
        }
    }

    pub fn advance_by_full_elapsed(&mut self) -> f64 {
        let elapsed = self.elapsed();
        if !(0.0..1.0).contains(&elapsed) {
            self.advance_by(elapsed.floor());
        }
        elapsed
    }
}



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

    pub fn elapsed(&self) -> bool {
        Instant::now() >= self.next
    }

    pub fn advance_if_elapsed(&mut self) -> bool {

        let now = Instant::now();

        if now >= self.next {
            let times = (now - self.next).div_duration_f64(self.duration).floor() + 1.0;
            self.next += self.duration.mul_f64(times);
            true
        }
        else { false }
    }

    pub fn advance(&mut self) {

        let now = Instant::now();

        if now > self.next {
            let times = (now - self.next).div_duration_f64(self.duration).ceil();
            self.next += self.duration.mul_f64(times);
        }
        else if self.next - now < self.duration {
            self.next += self.duration;
        }
        // else dont't do anything
    }
}



#[derive(Debug, Clone)]
pub struct IntervalCounter {
    pub count: u32,
    pub interval: StepInterval,
}

#[derive(Debug, Clone, Copy)]
pub struct IntervalCount {
    pub count: u32,
    pub times_per_sec: f32,
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
        if self.interval.advance_if_elapsed() {

            let counted = IntervalCount {
                times_per_sec: (self.count as f64 / self.interval.duration.div_duration_f64(Duration::SECOND)) as f32,
                count: self.count,
            };

            self.count = 0;

            Some(counted)
        }
        else { None }
    }
}



#[derive(Debug, Clone)]
pub struct AdaptRateInterval {
    pub next: Instant,
    pub duration: Duration,

    pub target_rate: f64,
    pub min_duration: Duration,

    pub count_interval: NormInterval,
    count: f64,
}

impl AdaptRateInterval {

    pub fn new(target_rate: f64, interval_duration: Duration, min_duration: Duration) -> Self {
        let duration = interval_duration.div_f64(target_rate);
        let next = Instant::now() + duration;
        Self {
            next, duration, target_rate, min_duration,
            count_interval: NormInterval::new(interval_duration), count: 0.0,
        }
    }

    pub fn from_per_sec(target_rate: f64, min_duration_ms: f64) -> Self {
        Self::new(target_rate, Duration::SECOND, Duration::from_secs_f64(min_duration_ms / 1e3))
    }

    pub fn elapsed(&self) -> bool {
        Instant::now() >= self.next
    }


    fn count(&mut self) {
        let elapsed = self.count_interval.advance_by_full_elapsed();
        if elapsed > 1.0 {
            if self.count > 0.0 && elapsed < 1.5 {
                self.duration = self.duration.mul_f64(self.count / self.target_rate);
            }
            self.count = 0.0;
        }
        self.count += 1.0;
    }


    pub fn advance(&mut self) {
        self.count();
        self.next = (self.next + self.duration).max(Instant::now() + self.min_duration);
    }


    pub fn advance_if_elapsed(&mut self) -> bool {
        let now = Instant::now();
        if now > self.next {
            self.count();
            self.next = (self.next + self.duration).max(Instant::now() + self.min_duration);
            true
        }
        else { false }
    }
}