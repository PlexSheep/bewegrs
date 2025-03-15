use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use sfml::cpp::FBox;
use sfml::system::Clock;
use sfml::SfResult;

pub type Ringbuffer<T, const SIZE: usize> = ConstGenericRingBuffer<T, SIZE>;

pub const MAX_FPS: u32 = 60;
pub const MAX_FPS_USIZE: usize = MAX_FPS as usize;

/// lazy fields get updated every [MAX_FPS] frames
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
    /// seconds from frame start to tick end for all frames of the second, so before the rendered
    /// stuff is made to be displayed
    pub frame_times: Ringbuffer<f32, MAX_FPS_USIZE>,
    /// actually keeps track of time
    pub clock: FBox<Clock>,
}

impl Counters {
    pub fn start() -> SfResult<Self> {
        Ok(Counters {
            clock: Clock::start()?,
            l_frames: 0,
            frames: 0,
            seconds: 0.0,
            l_seconds: 0.0,
            frame_times: Ringbuffer::new(),
        })
    }
    pub fn tick(&mut self) {
        self.seconds = self.clock.elapsed_time().as_seconds();
        self.frames += 1;

        if self.frames % MAX_FPS as u64 == 0 {
            println!("time passed: {:.2}s", self.seconds);
            println!("frames: {}", self.frames);
            println!("FPS: {}", self.fps());
            println!(
                "time per frame: {}ms / {}ms",
                self.a_frame_time(),
                1000.0 / MAX_FPS as f32
            );
            dbg!(&self.frame_times.len());
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
            eprintln!("dseconds 0");
            return MAX_FPS as f32; // only in the first second
        }
        self.dframes() as f32 / dseconds
    }

    pub fn a_frame_time(&self) -> f32 {
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    pub fn tick_done(&mut self) {
        self.frame_times
            .push((self.clock.elapsed_time().as_seconds() - self.seconds) * 1000.0);
    }
}
