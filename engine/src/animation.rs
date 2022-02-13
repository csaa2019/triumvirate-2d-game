use crate::image::*;

pub struct Animation {
    pub frames: i32,
    pub frame_duration: u64,
    pub first_sprite_index: usize,
    pub loops: bool,
}

impl Animation {
    pub fn new(frames: i32, frame_duration: u64, first_sprite_index: usize, loops: bool) {
        Animation {
            frames,
            frame_duration,
            first_sprite_index,
            loops,
        };
    }

    //to ask: this would have to return a value right? the current frame
    pub fn current_frame(&self, start_time: usize, now: usize, speedup_factor: usize) {
        //check what animation state we are currently at (this is the index that would choose which animation in the vector)

        //create something that would track time, call this "elapsed_time"

        //figure out which frame number we would be on based on elapsed time and animation.frame_duration
        //frame_number = (elapsed_time / animation.frame_duration) as i32 % animation.frames

        //using frame_number we get sprite_number
        //sprite_numnber = frame_number + animation.first_sprite_index

        //play_animation would use
    }
}

//To ask: could we use animation state as the thing that says which animation to use
pub struct AnimationState {
    pub sprite_index: i32, //not sure if its an i32
}

impl AnimationState {
    pub fn new(sprite_number: i32) {
        AnimationState {
            sprite_index: sprite_number,
        };
    }

    pub fn advance(&mut self) {
        self.sprite_index += 1;
    }
}

use std::rc::Rc;

pub struct Sprite {
    //source image for the sprite (chloe loaded it in as png)
    //To ask: why was this an Rc, and how/why we would use it
    pub image: Image,

    //why is this a vec of animations?
    pub animations: Vec<Animation>,

    //which animation state we are in: example, are we in running, rock, etc, etc.
    pub animation_state: AnimationState,

    //size of the sprite boxes on the sprite sheet --> should this be under the animation struct?
    pub sprite_size: Rect,
}

impl Sprite {
    //would we need a time thing here?
    pub fn play_animation() {
        //play an animation from the vec of animations
        //animation_state.sprite_index = animation.first_sprite_index + animation.current_frame()
        //draw(&self, ...., animation_state.sprite_index);
        //if loops,
    }

    // maybe some play_animation() function to start a new animation!
    // maybe some draw() function to draw the sprite!
    // we would use this draw function with the bitblt, right?
    pub fn draw(&self, fb: &mut Image, sprite_index: u32) {
        let from_rect = self.sprite_size; //this might change depending which sprite we draw from
        fb.bitblt(&self.image, &from_rect, (10, 10));
    }

    //advance the animation state (active frame)
    pub fn tick_animation(&mut self) {
        self.animation_state.advance();
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i);
}
