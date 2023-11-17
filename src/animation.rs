pub struct Animation {
    frames: Vec<[f32; 4]>,  // Assuming each frame is represented by [f32; 4]
    frame_duration: usize,   // Duration of each frame in milliseconds
}

pub struct AnimationState {
    current_frame: usize,
    elapsed_time: usize,
}

impl Animation {
    pub fn new(frames: Vec<[f32; 4]>, frame_duration: usize) -> Self {
        Animation { frames, frame_duration }
    }

    pub fn sample(&self, start_time: usize, now: usize, speedup_factor: usize) -> [f32; 4] {
        let elapsed_time = now - start_time;
        let frame_index = (elapsed_time / (self.frame_duration / speedup_factor)) % self.frames.len();

        self.frames[frame_index]
    }
}

impl AnimationState {
    pub fn new() -> Self {
        AnimationState {
            current_frame: 0,
            elapsed_time: 0,
        }
    }

    pub fn tick(&mut self, elapsed_time: usize) {
        self.elapsed_time += elapsed_time;
        // Optionally, you can add logic to handle animation loops or transitions
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }
}