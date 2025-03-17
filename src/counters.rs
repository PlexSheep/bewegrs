use std::fmt::Write;

use ringbuffer::RingBuffer as _;
use sfml::SfResult;
use sfml::cpp::FBox;
use sfml::system::Clock;

type RingBuffer<T> = ringbuffer::AllocRingBuffer<T>;

/// lazy fields get updated every [Self::fps_limit] frames
#[derive(Debug)]
pub struct Counters {
    /// frame counter
    pub frames: u64,
    /// frame counter lazy
    pub l_frames: u64,
    /// seconds counter
    pub seconds: f32,
    /// seconds counter lazy
    pub l_seconds: f32,
    pub frame_time_pre: f32,
    pub frame_times: RingBuffer<f32>,
    /// actually keeps track of time
    pub clock: FBox<Clock>,
    pub fps_limit: u64,

    pub text: String,
}

impl Counters {
    // pub const MS_PER_FRAME: f32 = 1000.0 / MAX_FPS as f32;
    // pub const MAX_FPS_U64: u64 = MAX_FPS as u64;

    pub fn start(fps_limit: u64) -> SfResult<Self> {
        let mut c = Counters {
            clock: Clock::start()?,
            l_frames: 0,
            frames: 0,
            seconds: 0.0,
            l_seconds: 0.0,
            frame_time_pre: 0.0,
            frame_times: RingBuffer::new(fps_limit as usize),
            text: String::new(),
            fps_limit,
        };
        c.update_text();
        Ok(c)
    }

    #[inline]
    pub const fn ms_per_frame(&self) -> f32 {
        1000.0 / self.fps_limit as f32
    }

    pub fn update_text(&mut self) {
        self.text.clear();
        writeln!(self.text, "time passed: {:.2}s", self.seconds)
            .expect("could not write to text buffer");
        writeln!(self.text, "frames: {}", self.frames).expect("could not write to text buffer");

        writeln!(self.text, "FPS: {:02.1}", self.fps().round())
            .expect("could not write to text buffer");
        write!(
            self.text,
            "time per frame: {:02.2}ms / {:02.2}ms",
            self.a_frame_time(),
            self.ms_per_frame()
        )
        .expect("could not write to text buffer");
    }

    pub fn frame_start(&mut self) {
        self.seconds = self.clock.elapsed_time().as_seconds();
        self.frames += 1;

        if self.frames % self.fps_limit == 0 || self.frames == 1 {
            self.update_text();
            self.l_seconds = self.seconds;
            self.l_frames = self.frames;
        }
    }

    pub fn dframes(&self) -> u64 {
        self.frames - self.l_frames
    }

    pub fn dseconds(&self) -> f32 {
        self.seconds - self.l_seconds
    }

    pub fn fps(&self) -> f32 {
        let dseconds = self.dseconds();
        if dseconds == 0.0 {
            return self.fps_limit as f32; // only in the first second
        }
        self.dframes() as f32 / dseconds
    }

    pub fn a_frame_time(&self) -> f32 {
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    pub fn frame_prepare_display(&mut self) {
        self.frame_times
            .push((self.clock.elapsed_time().as_seconds() - self.seconds) * 1000.0);
    }
}
