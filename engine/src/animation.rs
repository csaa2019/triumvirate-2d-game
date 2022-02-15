use crate::image::*;

pub struct Animation {
    //vec of frames from the sprite sheet that are a part of this animation
    //for example, frame 0, 5, 6 for a "running" animation
    pub frames: Vec<usize>,

    //duration of each frame
    pub frame_duration: u64,

    //whether or not this animation loops or not
    pub loops: bool,

    //size of each frame on the sprite sheet for this animation
    pub sprite_size: Rect,
}

impl Animation {
    pub fn new(frames: Vec<usize>, frame_duration: u64, loops: bool) {
        Animation {
            frames,
            frame_duration,
            loops,
        };
    }
}

pub struct AnimationState {
    //elapsed time
    pub elapsed_time: usize,

    //the currently active frame, also the sprite index
    pub current_frame: usize,

    //which animation state we are currently in
    //i. e. "running, walking, etc."
    pub animation_index: usize,
}

impl AnimationState {
    pub fn new(current_frame: usize, elapsed_time: usize, animation_index: usize) {
        AnimationState {
            current_frame: current_frame,
            elapsed_time,
            animation_index,
        };
    }

    pub fn advance(&mut self) {
        self.current_frame += 1;
    }

    //takes animation state information and produces a rect that we can use in the bitblt function
    pub fn current_frame(&self, start_time: usize, now: usize, speedup_factor: usize) -> Rect {
        //check what animation state we are currently at (this is the index that would choose which animation in the vector)

        //create something that would track time, call this "elapsed_time"

        //figure out which frame number we would be on based on elapsed time and animation.frame_duration
        //frame_number = (elapsed_time / animation.frame_duration) as i32 % animation.frames

        //using frame_number we get sprite_number
        //sprite_numnber = frame_number + animation.first_sprite_index

        //play_animation would use
    }
}

use std::rc::Rc;

//spritesheet struct here --> that has the size of the rectangles here

pub struct Sprite {
    //To ask: why was this an Rc, and how/why we would use it
    //answer: if we have 5 Sprites that have the same image, we don't want to load the same image
    //every single time that we use it, this is why we use Rc
    // alternatively could use somethign that utilizes lifetimes: Sprite<'img> { image: &'img Image }
    pub image: Rc<Image>,

    //animations contained on this sprite sheet
    pub animations: Vec<Animation>,

    //what is the currently active frame, what animation are we using
    pub animation_state: AnimationState,
}

//engine will update the animatino state of the sprite, tick it forward, have an if statement that checks if it loops
//and if it has ended

impl Sprite {
    //would we need a time thing here?
    pub fn play_animation(&self, animation_index: usize) {
        self.animation_state = 0;
        //draw will pick the right rectangle based ont eh current animation and animation state

        //play an animation from the vec of animations
        //animation_state.sprite_index = animation.first_sprite_index + animation.current_frame()
        //draw(&self, ...., animation_state.sprite_index);
        //if loops,
    }
    pub fn draw(&self, fb: &mut Image, sprite_index: u32) {
        let from_rect = self.animations[animation_state.animation_index].sprite_size;
        fb.bitblt(&self.image, &from_rect, (10, 10));
    }

    //Grace: I don't think we need this tick_animatino thing here, confirm on tuesday
    //advance the animation state (active frame)
    pub fn tick_animation(&mut self) {
        self.animation_state.advance();
    }
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i);
}
