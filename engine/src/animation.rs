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

    // number of sprites wide
    pub sprite_width: u32,

    // number of sprites high
    pub sprite_height: u32,
}

impl Animation {
    pub fn new(
        frames: Vec<usize>,
        frame_duration: u64,
        loops: bool,
        sprite_size: Rect,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Animation {
        Animation {
            frames,
            frame_duration,
            loops,
            sprite_size,
            sprite_width,
            sprite_height,
        }
    }
}
pub struct AnimationState {
    //elapsed time
    pub elapsed_time: usize,

    //the INDEX of the currently active frame in animation.frames
    pub current_frame: usize,

    //which animation state we are currently in
    //i. e. "running, walking, etc.", this we pass into the vec<animations> of sprite
    pub animation_index: usize,
}

impl AnimationState {
    pub fn new(
        current_frame: usize,
        elapsed_time: usize,
        animation_index: usize,
    ) -> AnimationState {
        AnimationState {
            current_frame: current_frame,
            elapsed_time,
            animation_index,
        }
    }

    pub fn advance(&mut self, animations: &Vec<Animation>) {
        let frame_rate = animations[self.animation_index].frame_duration;
        let loops = animations[self.animation_index].loops;

        if (self.elapsed_time == frame_rate as usize) {
            self.current_frame += 1;
            if self.current_frame >= animations[self.animation_index].frames.len() {
                self.current_frame = 0;
            }
            if (loops) {
                self.elapsed_time = 0;
            }
        }
        self.elapsed_time += 1;
    }

    pub fn change_animation_index(&mut self, new_index: usize) {
        self.animation_index = new_index;
    }
    pub fn get_animation_index(&self) -> usize {
        self.animation_index
    }

    // takes animation state information and produces a rect that we can use in the bitblt function
    //    pub fn current_frame(&self, start_time: usize, now: usize, speedup_factor: usize) -> Rect {

    pub fn current_frame(&self, animations: &Vec<Animation>) -> Rect {
        //check what animation state we are currently at (this is the index that would choose which animation in the vector)
        let animation = &animations[self.animation_index];
        //create something that would track time, call this "elapsed_time"

        let mut sprite_rect = animation.sprite_size;
        let sprite_index = animations[self.animation_index].frames[self.current_frame];

        let w = animation.sprite_width;
        let h = animation.sprite_height;
        assert!((sprite_index as u32) < (w * h));

        if sprite_index == 0 {
            return sprite_rect;
        }

        let column = sprite_index as u32 % w;
        let row = sprite_index as u32 / w;

        // FOR DEBUGGING
        // print!(
        //     "{} {} {} {} {} {}\n",
        //     w, sprite_index, column, row, sprite_rect.x0, sprite_rect.y0
        // );
        sprite_rect.x0 = sprite_rect.x0 + (column * sprite_rect.w) as i32;
        sprite_rect.y0 = sprite_rect.y0 + (row * sprite_rect.h) as i32;

        return sprite_rect;
    }

    //old code:
    //figure out which frame number we would be on based on elapsed time and animation.frame_duration
    //frame_number = (elapsed_time / animation.frame_duration) as i32 % animation.frames
    //using frame_number we get sprite_number
    //sprite_numnber = frame_number + animation.first_sprite_index
    //play_animation would use
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
    pub fn play_animation(&mut self, fb: &mut Image, to: Vec2i) {
        self.animation_state.current_frame = 0;
        self.animation_state.elapsed_time = 0;
        self.animation_state.animation_index = self.animation_state.animation_index;

        // find current frame

        // draw the current frame (this takes the rectangle)

        //draw will pick the right rectangle based ont eh current animation and animation state
        self.draw(fb, to);
        //play an animation from the vec of animations
        //animation_state.sprite_index = animation.first_sprite_index + animation.current_frame()
        //draw(&self, ...., animation_state.sprite_index);
        //if loops,
    }

    pub fn draw(&self, fb: &mut Image, to: Vec2i) {
        let sprite_rect = self.animation_state.current_frame(&self.animations);

        fb.bitblt(&self.image.as_ref(), &sprite_rect, to);
    }

    pub fn change_animation(&mut self, frame: usize) {
        self.animation_state.current_frame = frame;
    }

    pub fn draw_specific_frame(&mut self, fb: &mut Image, to: Vec2i, frame: usize) {
        self.animation_state.current_frame = frame;
        let sprite_rect = self.animation_state.current_frame(&self.animations);

        fb.bitblt(&self.image.as_ref(), &sprite_rect, to);
    }

    // pub fn draw(&self, fb: &mut Image) {
    //     print!("Getting Sprite Rect");
    //     let sprite_rect = self.animation_state.current_frame(&self.animations);

    //     print!("bitbltting");

    //     fb.bitblt(&self.image.as_ref(), &sprite_rect, (10, 10));
    // }

    //advance the animation state (active frame)

    //tick animation at the right pace, should have a parameter here
    pub fn tick_animation(&mut self) {
        self.animation_state.advance(&self.animations);
    }
}

// pub trait DrawSpriteExt {
//     fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i);
// }

pub struct Font {
    pub font_sprite: Sprite,
    pub horizontal_spacing: u32,
    pub font_width: u32,
    pub font_height: u32,
}

pub fn get_fonts() -> (Font, Font) {
    let fontsize = (7 as f32 * 1.5) as u32;
    let fontsize_h = (8 as f32 * 1.5) as u32;
    let fontsheet_w = 16 * fontsize;
    let fontsheet_h = 8 * fontsize_h;

    // the rectangle of one sprite
    let font_sprite_rect = Rect::new(0, 0, fontsize, fontsize_h);

    let fontsheet =
        Image::from_png_not_premultiplied("content/fontsheet_7x8.png", fontsheet_w, fontsheet_h);

    let font_anim_state = AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };

    let font_anim_1 = Animation {
        frames: (0..128).collect(),
        frame_duration: 10,
        loops: true,
        sprite_size: font_sprite_rect,
        sprite_width: 16,
        sprite_height: 8,
    };

    let mut fontsheet_sprite = Sprite {
        image: Rc::new(fontsheet),
        animations: vec![font_anim_1],
        animation_state: font_anim_state,
    };

    let titlefont_size = 28;
    let titlefont_size_height = 32;

    let titlefontsheet_w = 16 * titlefont_size;
    let titlefontsheet_h = 8 * titlefont_size_height;

    // the rectangle of one sprite
    let titlefont_sprite_rect = Rect::new(0, 0, titlefont_size, titlefont_size_height);

    let titlefontsheet = Image::from_png_not_premultiplied(
        "content/fontsheet_70x80.png",
        titlefontsheet_w,
        titlefontsheet_h,
    );

    let titlefont_anim_state = AnimationState {
        current_frame: 0,
        elapsed_time: 0,
        animation_index: 0,
    };

    let titlefont_anim_1 = Animation {
        frames: (0..128).collect(),
        frame_duration: 10,
        loops: true,
        sprite_size: titlefont_sprite_rect,
        sprite_width: 16,
        sprite_height: 8,
    };

    let mut titlefontsheet_sprite = Sprite {
        image: Rc::new(titlefontsheet),
        animations: vec![titlefont_anim_1],
        animation_state: titlefont_anim_state,
    };

    (
        Font {
            font_sprite: titlefontsheet_sprite,
            horizontal_spacing: (titlefont_size as f32 * 0.6) as u32,
            font_width: titlefont_size,
            font_height: titlefont_size_height,
        },
        Font {
            font_sprite: fontsheet_sprite,
            horizontal_spacing: (fontsize as f32 * 0.8) as u32,
            font_width: fontsize,
            font_height: fontsize_h,
        },
    )
}
