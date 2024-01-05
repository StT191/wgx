
pub use std::time::{Instant, Duration};

#[derive(Debug)]
pub struct Profile {
  pub name: &'static str,
  duration: Duration, next_measure: Option<Instant>,
  count: u32,
  amount: Duration, min: Duration, max: Duration,
}

#[derive(Debug)]
pub struct Measure {
  pub name: &'static str,
  pub count: u32,
  pub average: Duration, pub min: Duration, pub max: Duration,
}

impl Profile {

  pub const fn new(name: &'static str, duration: Duration) -> Self {
    Self {
      name, duration, next_measure: None, count: 0,
      amount: Duration::from_nanos(0), max: Duration::from_nanos(0), min: Duration::MAX,
    }
  }

  pub fn measure(&mut self, value: Duration) -> Option<Measure> {

    // measure
    let now = Instant::now();
    let mut measure = None;

    if let Some(next_time) = self.next_measure {
      if now > next_time {

        measure = Some(Measure {
          name: self.name, count: self.count,
          average: self.amount / self.count,
          max: self.max, min: self.min,
        });

        // reset
        self.count = 0;
        self.amount = Duration::from_nanos(0);
        self.max = Duration::from_nanos(0);
        self.min = Duration::MAX;
        self.next_measure = Some(now + self.duration);
      }
    }
    else {
      self.next_measure = Some(now + self.duration);
    }

    // count
    self.count += 1;
    self.amount += value;

    if value > self.max { self.max = value }
    if value < self.min { self.min = value }

    // return
    measure
  }
}

#[macro_export]
macro_rules! unsafe_measure_log {
  ($profile:expr, $value:expr, $log:path) => {
    unsafe {
      if let Some(measure) = $profile.measure($value) {
        $log!("{:?}", measure);
        true
      }
      else { false }
    }
  }
}


#[macro_export]
macro_rules! profile {
  ($name:expr, $duration:expr, $log:path, $code:expr) => {{

    use std::time::Instant;
    use profile::*;

    static mut PROFILE: Profile = Profile::new($name, $duration);
    let then = Instant::now();

    $code;

    $crate::unsafe_measure_log!(PROFILE, then.elapsed(), $log)
  }}
}