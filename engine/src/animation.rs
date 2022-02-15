use crate::image::*;

pub struct Animation {
    //vec of frames from the sprite sheet that are a part of this animation
    //for example, frame 0, 5, 6 for a "running" animation
    pub frames: Vec<usize>,

    //duration of each frame
    pub frame_duration: u64,

    //whether or not this animation loops or not
    pub loops: bool,
}

impl Animation {
    pub fn new(frames: Vec<usize>, frame_duration: u64, first_sprite_index: usize, loops: bool) {
        Animation {
            frames,
            frame_duration,
            //first_sprite_index,
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

pub struct AnimationState {
    //elapsed time
    pub elapsed_time: usize,
    //instead of having sprite_index we would have something here called elapsed_time
    pub sprite_index: usize, //not sure if its an i32
    //storing elapsed time here
    //could compute the index of which animation we are right now

    //which animation state we are currently in
    pub animation_index: usize,
}

impl AnimationState {
    pub fn new(sprite_index: usize, elapsed_time: usize, animation_index: usize) {
        AnimationState {
            sprite_index,
            elapsed_time,
            animation_index,
        };
    }

    pub fn advance(&mut self) {
        self.sprite_index += 1;
    }
}

use std::rc::Rc;

//spritesheet struct here --> that has the size of the rectangles here

pub struct Sprite {
    //source image for the sprite (chloe loaded it in as png)
    //To ask: why was this an Rc, and how/why we would use it
    //answer: if we have 5 psprites that have the same image, we don't want to load the same image
    //every single time that we use it, this is why we use Rc
    // alternatively could use somethign that utilizes lifetimes: Sprite<'img> { image: &'img Image }
    pub image: Rc<Image>,

    pub animations: Vec<Animation>,

    pub animation_state: AnimationState,

    //size of the sprite boxes on the sprite sheet --> should this be under the animation struct?
    pub sprite_size: Rect,
    //pub first_sprite_index: usize,
}

//engine will update the animatino state of the sprite, tick it forward, have an if statement that checks if it loops
//and if it has ended

impl Sprite {
    //would we need a time thing here?
    pub fn play_animation(&self, animation_index: usize) {
        self.animation_state = 0;
        //producing a new animation state self.animation state == something
        //draw will pick the right rectangle based ont eh current animation and animation state

        //play an animation from the vec of animations
        //animation_state.sprite_index = animation.first_sprite_index + animation.current_frame()
        //draw(&self, ...., animation_state.sprite_index);
        //if loops,
    }
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
