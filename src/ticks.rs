
use instant::{Instant, Duration};


#[derive(Debug)]
pub struct TickTimer {
    pub tick_duration: Duration,
    pub min_duration: Duration,
    pub last_delta_time: Duration,
    pub next_tick: Instant,
}

#[derive(Debug)]
pub struct TickShift {
    pub was_next_tick: bool,
    pub delta_time: Duration,
}

impl TickShift {
    pub fn new() -> Self {
        Self {
            was_next_tick: false,
            delta_time: Duration::from_millis(0),
        }
    }
}

impl TickTimer {

    pub fn new(tick_duration:Duration, min_duration:Duration) -> Self {
        Self {
            tick_duration,
            min_duration: min_duration,
            last_delta_time: Duration::from_nanos(0),
            next_tick: Instant::now() + tick_duration,
        }
    }

    pub fn from_ticks_per_sec(ticks_per_sec:f64, min_duration_millis:f64) -> Self {
        Self::new(Duration::from_secs_f64(1.0 / ticks_per_sec), Duration::from_secs_f64(min_duration_millis / 1000.0))
    }

    pub fn elapsed(&self) -> bool {
        Instant::now() >= self.next_tick
    }

    pub fn next(&mut self) -> TickShift {

        let now = Instant::now();

        let mut shift = TickShift::new();

        if now >= self.next_tick {

            shift.was_next_tick = true;
            shift.delta_time = now - self.next_tick;

            if self.tick_duration > shift.delta_time + self.min_duration {
                self.last_delta_time = shift.delta_time;
            }
        }
        else {
            shift.delta_time = self.next_tick - now;
        }

        // next tick
        self.next_tick = now + self.tick_duration;

        if self.tick_duration > self.last_delta_time {
            self.next_tick -= self.last_delta_time;
        }

        shift
    }
}



#[derive(Debug)]
pub struct TickCounter {
    pub count_time: Duration,
    pub last_time: Instant,

    pub ticks: u32,
    pub deltas: u32,
    pub delta_times_sum: Duration,
}

#[derive(Debug)]
pub struct TickCount {
    pub ticks_per_sec: f32,
    pub average_delta_time: Duration,
}

impl TickCounter {

    pub fn new(count_time:Duration) -> Self {
        Self {
            count_time,
            last_time: Instant::now(),
            ticks: 0, deltas: 0,
            delta_times_sum: Duration::from_micros(0),
        }
    }

    pub fn from_secs(count_secs:f64) -> Self {
        Self::new(Duration::from_secs_f64(count_secs))
    }

    pub fn add_one(&mut self) {
        self.ticks += 1;
    }

    pub fn add_delta_time(&mut self, delta_time:Duration) {
        self.delta_times_sum += delta_time;
        self.deltas += 1;
    }

    pub fn tick_count(&mut self) -> Option<TickCount> {

        let elapsed = self.last_time.elapsed();

        if elapsed >= self.count_time {
            self.last_time = Instant::now();

            let count = TickCount {
                ticks_per_sec: self.ticks as f32 / elapsed.as_secs_f32(),
                average_delta_time: self.delta_times_sum / self.deltas,
            };

            self.ticks = 0;
            self.deltas = 0;
            self.delta_times_sum = Duration::from_micros(0);

            return Some(count);
        }

        None
    }
}
