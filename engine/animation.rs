pub struct Animation {
    // Do this for the exercise today!
    // You'll want to know the frames involved and the timing for each frame
}

pub struct AnimationState {
    // Here you'll need to track how far along in the animation you are.
    // You can also choose to have an Rc<Animation> to refer to the animation in use.
    // But you could also decide to just pass around the animation and state together
    // where needed.
    // Could be ticked in-place with a function like tick(&mut self)
}

impl Animation {
    // Should hold some data...
    // Be used to decide what frame to use...
    // Could have a query function like sample(&self, start_time:usize, now:usize, speedup_factor:usize) -> [f32;4]
}