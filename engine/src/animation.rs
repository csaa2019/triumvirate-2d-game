pub struct Animation {

    pub frames: i32, 
    pub frame_duration: u64, 
    pub first_sprite_index: usize, 

    impl Component for Animation{
        type Storage = DenseVecStorage<Self>; 
    }
    // Do this for the exercise today!
// You'll want to know the frames involved and the timing for each frame
}

pub struct AnimationState {

    //would t 
    // Here you'll need to track how far along in the animation you are.
// You can also choose to have an Rc<Animation> to refer to the animation in use.
// But you could also decide to just pass around the animation and state together
// where needed.
}

impl Animation {
    fn new()

  
    // Should hold some data...
    // Be used to decide what frame to use...
    // Could have a query function like current_frame(&self, start_time:usize, now:usize, speedup_factor:usize)
    // Or could be ticked in-place with a function like tick(&self)
}

use std::rc::Rc;

pub struct Sprite {
    image: Rc<Image>,
    // For example, but this is just one way to do it:
    animations:Vec<animation::Animation>,
    animation_state:animation::AnimationState,
}

impl Sprite {

    fn play_animation(){

    }
    // maybe some play_animation() function to start a new animation!
    // maybe some draw() function to draw the sprite!

    fn tick_animation(){

    }

    // and a tick_animation() function to advance the animation state
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite, pos:Vec2i);
}
