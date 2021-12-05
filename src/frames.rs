
use std::{time::{Instant, Duration}};


#[derive(Debug)]
pub struct FrameTimer {
    pub frame_time: Duration,
    pub min_frame_time: Duration,
    pub last_delta_time: Duration,
    pub next_redraw: Instant,
}

#[derive(Debug)]
pub struct FrameShift {
    pub was_next_frame: bool,
    pub delta_time: Duration,
}

impl FrameShift {
    pub fn new() -> Self {
        Self {
            was_next_frame: false,
            delta_time: Duration::from_millis(0),
        }
    }
}

impl FrameTimer {

    pub fn from_frame_rate(frame_rate:u64, min_frame_millis:u64) -> Self {

        let frame_time = Duration::from_micros(1_000_000 / frame_rate);

        Self {
            frame_time,
            min_frame_time: Duration::from_millis(min_frame_millis),
            last_delta_time: Duration::from_micros(0),
            next_redraw: Instant::now() + frame_time,
        }
    }

    pub fn needs_redraw(&self) -> bool {
        Instant::now() >= self.next_redraw
    }

    pub fn next(&mut self) -> FrameShift {

        let now = Instant::now();

        let mut shift = FrameShift::new();

        if now >= self.next_redraw {

            shift.was_next_frame = true;
            shift.delta_time = now - self.next_redraw;

            if self.frame_time > shift.delta_time + self.min_frame_time {
                self.last_delta_time = shift.delta_time;
            }
        }

        self.next_redraw = now + self.frame_time - self.last_delta_time;

        return shift;
    }
}



#[derive(Debug)]
pub struct FrameCounter {
    pub count_time: Duration,
    pub last_time: Instant,

    pub frames: u32,
    pub deltas: u32,
    pub delta_times_sum: Duration,
}

#[derive(Debug)]
pub struct FrameCount {
    pub frames_per_sec: f32,
    pub average_delta_time: Duration,
}

impl FrameCounter {

    pub fn from_secs(count_time:f32) -> Self {
        Self {
            count_time: Duration::from_secs_f32(count_time),
            last_time: Instant::now(),
            frames: 0, deltas: 0,
            delta_times_sum: Duration::from_micros(0),
        }
    }

    pub fn add_frame(&mut self) {
        self.frames += 1;
    }

    pub fn add_delta_time(&mut self, delta_time:Duration) {
        self.delta_times_sum += delta_time;
        self.deltas += 1;
    }

    pub fn tick(&mut self) -> Option<FrameCount> {

        let elapsed = self.last_time.elapsed();

        if elapsed >= self.count_time {
            self.last_time = Instant::now();

            let count = FrameCount {
                frames_per_sec: self.frames as f32 / elapsed.as_secs_f32(),
                average_delta_time: self.delta_times_sum / self.deltas,
            };

            self.frames = 0;
            self.deltas = 0;
            self.delta_times_sum = Duration::from_micros(0);

            return Some(count);
        }

        None
    }
}
