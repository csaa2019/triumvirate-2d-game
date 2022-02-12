use crate::image::*;

pub struct Animation {
    pub frames: i32,
    pub frame_duration: u64,
    pub first_sprite_index: usize,
}
// Do this for the exercise today!
// You'll want to know the frames involved and the timing for each frame

impl Animation {
    pub fn new(frames: i32, frame_duration: u64, first_sprite_index: usize) {
        Animation {
            frames,
            frame_duration,
            first_sprite_index,
        };
    }

    pub fn current_frame(&self, start_time: usize, now: usize, speedup_factor: usize) {}

    // Should hold some data...
    // Be used to decide what frame to use...
    // Could have a query function like current_frame(&self, start_time:usize, now:usize, speedup_factor:usize)
    // Or could be ticked in-place with a function like tick(&self)
}

pub struct AnimationState {
    pub sprite_number: i32, //not sure if its an i32
}

impl AnimationState {
    pub fn new(sprite_number: i32) {
        AnimationState { sprite_number };
    }

    pub fn advance(&mut self) {
        self.sprite_number += 1;
    }
}

use std::rc::Rc;

pub struct Sprite {
    //source image for the sprite (chloe loaded it in as png)
    pub image: Image,
    // ASK WHY AND HOW TO USE RC INSTEAD?
    // For example, but this is just one way to do it:
    pub animations: Vec<Animation>,
    //why is this a vec of animations?
    pub animation_state: AnimationState,

    pub sprite_size: Rect,
}

impl Sprite {
    //would we need a time thing here?
    pub fn play_animation() {
        //need to track time, variable "elapsed_time"
        //using elapsed_time, we will figure out which frame number we are on: frame_number
        //frame_number = (elapsed_time / animation.frame_duration) as i32 % animation.frames
        //using frame_number, we will get sprite_number
        //sprite_numnber = frame_number + animation.first_sprite_index
    }

    // maybe some play_animation() function to start a new animation!
    // maybe some draw() function to draw the sprite!
    // we would use this draw function with the bitblt, right?
    pub fn draw(&self, fb: &mut Image, sprite_index: u32) {
        let from_rect = self.sprite_size; //this might change depending which sprite we draw from
        fb.bitblt(&self.image, &from_rect, (10, 10));
    }

    // and a tick_animation() function to advance the animation state
    pub fn tick_animation(&mut self) {
        self.animation_state.advance();
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i);
}
