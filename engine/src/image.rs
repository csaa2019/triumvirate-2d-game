use crate::animation::*;
use image::Triangle;
use itertools::Itertools;

pub type Color = (u8, u8, u8, u8);

// const WIDTH: usize = 320;
// const HEIGHT: usize = 240;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec2i {
    // Or Vec2f for floats?
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32, // (x0, y0) is the left upper corner
    pub w: u32,
    pub h: u32, // (x1, y1) is the bottom right corner
}

pub trait DrawSpriteExt {
    fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i);
}

// impl DrawSpriteExt for Image {
//     fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i) {
//         // This works because we're only using a public method of Screen here,
//         // and the private fields of Sprite are visible inside this module
//         self.bitblt(&s.image, s.animation_state.current_frame(), pos);
//     }
// }

impl Rect {
    pub fn new(x0: i32, y0: i32, w: u32, h: u32) -> Rect {
        Rect { x0, y0, w, h }
    }
    // TODO: Add rect_inside function which will check if a point is inside the rect.

    pub fn rect_inside(&self, p: Vec2i) -> bool {
        let x1 = self.x0 + self.w as i32;
        let y1 = self.y0 + self.h as i32;

        if p.x > self.x0 && p.x < x1 {
            if p.y > self.y0 && p.y < y1 {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
    // This is some changing height and width code in case we want it.

    // pub fn change_h(&mut self, inc: i32) {
    //     let h = (self.h as i32) + inc;
    //     if (self.y0 + h <= (HEIGHT - 2) as i32) && h > 4 {
    //         self.h = h as u32;
    //     }
    // }

    // pub fn change_w(&mut self, inc: i32) {
    //     let w = (self.w as i32) + inc;
    //     if (self.x0 + w <= (WIDTH - 2) as i32) && w > 4 {
    //         self.w = w as u32;
    //     }
    // }
}
/**
 * This is basically a struct that contains a framebuffer.
 * It also contains a width and a height.
 */
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Image {
    pub buffer: Box<[Color]>, // or Vec<Color>, or...
    pub w: usize,
    pub h: usize,
}

impl Image {
    pub fn new(c: Color, w: usize, h: usize) -> Image {
        Image {
            buffer: vec![c; w * h].into_boxed_slice(),
            w,
            h,
        }
    }

    /**
     * This function will create an Image if you give it a filename, a width, and a height.
     * This can be used to load in textures, spritesheets, etc.
     */
    pub fn from_png(filename: &str, img_width: u32, img_height: u32) -> Image {
        let (image_data, dimensions) = {
            let img = image::open(filename).unwrap();
            let img_resized = image::imageops::resize(&img, img_width, img_height, Triangle);
            let dim = img_resized.dimensions();
            (img_resized.into_raw(), dim)
        };
        let mut texture = Vec::<Color>::new();

        for (a, b, c, d) in image_data.into_iter().tuples() {
            let c: Color = (a, b, c, d);
            texture.push(c);
        }

        Image {
            buffer: texture.into_boxed_slice(),
            w: img_width as usize,
            h: img_height as usize,
        }
    }

    pub fn clear(&mut self, c: Color) {
        self.buffer.fill(c);
    }

    pub fn as_slice(&self) -> &[Color] {
        &self.buffer
    }

    /**
     * This is the bitblt function.
     * It is a method called on an Image struct.
     * Ideally, you call this on your framebuffer.
     * You pass it a src_img, which is ALSO an image struct. This is the texture.
     * The function above, from_png, will make you an Image from a file you pass to it
     * and the dimensions you specify.
     *
     * The from &rect is the part of the source image you want to copy over.
     * The to (i32, i32) is the coordinates of where to paste this over to in the framebuffer.
     *
     * For example, in my code, this is how I called bitblt:
     *       fb2d.bitblt(&texture, &new_rect, (10, 10));
     *
     * NOTE: The size of your texture (wxh) should be the SAME as your rect's wxh.
     * This can be achieved by passing the rectangle's wxh into from_png
     * when creating your texture Image. If they are not the same, the texture will get cut off.
     *
     * (Nate or Grace, maybe you could figure out a way to just make it resize?)
     */
    pub fn bitblt(&mut self, src_img: &Image, from: &Rect, to: Vec2i) {
        let dst = &mut self.buffer;
        let dst_size = (self.w, self.h);
        let src = src_img.as_slice();
        let src_size = (src_img.w, src_img.h);

        // implement after writing a rect_inside
        // assert!(rect_inside(from, (0, 0, src_size.0, src_size.1)));
        let Vec2i { x: to_x, y: to_y } = to;
        if (to_x + from.w as i32) < 0
            || (dst_size.0 as i32) <= to_x
            || (to_y + from.h as i32) < 0
            || (dst_size.1 as i32) <= to_y
        {
            return;
        }
        let src_pitch = src_size.0;
        let dst_pitch = dst_size.0;
        // All this rigmarole is just to avoid bounds checks on each pixel of the blit.
        // We want to calculate which row/col of the src image to start at and which to end at.
        // This way there's no need to even check for out of bounds draws---
        // we'll skip rows that are off the top or off the bottom of the image
        // and skip columns off the left or right sides.
        let y_skip = to_y.max(0) - to_y;
        let x_skip = to_x.max(0) - to_x;
        let y_count = (to_y + from.h as i32).min(dst_size.1 as i32) - to_y;
        let x_count = (to_x + from.w as i32).min(dst_size.0 as i32) - to_x;
        // The code above is gnarly so these are just for safety:
        debug_assert!(0 <= x_skip);
        debug_assert!(0 <= y_skip);
        debug_assert!(0 <= x_count);
        debug_assert!(0 <= y_count);
        debug_assert!(x_count <= from.w as i32);
        debug_assert!(y_count <= from.h as i32);
        debug_assert!(0 <= to_x + x_skip);
        debug_assert!(0 <= to_y + y_skip);
        debug_assert!(0 <= from.x0 as i32 + x_skip);
        debug_assert!(0 <= from.y0 as i32 + y_skip);
        debug_assert!(to_x + x_count <= dst_size.0 as i32);
        debug_assert!(to_y + y_count <= dst_size.1 as i32);
        // OK, let's do some copying now
        for (row_a, row_b) in src
        // From the first pixel of the top row to the first pixel of the row past the bottom...
            [(src_pitch * (from.y0 as i32 + y_skip) as usize)..(src_pitch * (from.y0 as i32 + y_count) as usize)]
        // For each whole row...
            .chunks_exact(src_pitch)
        // Tie it up with the corresponding row from dst
            .zip(
                dst[(dst_pitch * (to_y + y_skip) as usize)
                    ..(dst_pitch * (to_y + y_count) as usize)]
                    .chunks_exact_mut(dst_pitch),
            )
        {
            // Get column iterators, save on indexing overhead
            let to_cols = row_b
                [((to_x + x_skip) as usize)..((to_x + x_count) as usize)].iter_mut();
            let from_cols = row_a
                [((from.x0 as i32 + x_skip) as usize)..((from.x0 as i32 + x_count) as usize)].iter();
            // Composite over, assume premultiplied rgba8888 in src!
            for (to, from) in to_cols.zip(from_cols) {
                let ta = to.3 as f32 / 255.0;
                let fa = from.3 as f32 / 255.0;
                to.0 = from.0.saturating_add((to.0 as f32 * (1.0 - fa)).round() as u8);
                to.1 = from.1.saturating_add((to.1 as f32 * (1.0 - fa)).round() as u8);
                to.2 = from.2.saturating_add((to.2 as f32 * (1.0 - fa)).round() as u8);
                to.3 = ((fa + ta * (1.0 - fa)) * 255.0).round() as u8;
            }
        }
    }

    // THE FOLLOWING CODE IS MY STUFF FROM THE RECTANGLE LAB
    // MIGHT BE USEFUL FOR THE GAME??

    // #[allow(dead_code)]
    // fn hline(&mut self, x0: usize, x1: usize, y: usize, c: Color) {
    //     let fb = &mut self.buffer;
    //     assert!(y < HEIGHT);
    //     assert!(x0 <= x1);
    //     assert!(x1 < WIDTH);
    //     fb[y * WIDTH + x0..(y * WIDTH + x1)].fill(c);
    // }
    // #[allow(dead_code)]
    // fn line(&mut self, (x0, y0): (usize, usize), (x1, y1): (usize, usize), col: Color) {
    //     let fb = &mut self.buffer;
    //     let mut x = x0 as i64;
    //     let mut y = y0 as i64;
    //     let x0 = x0 as i64;
    //     let y0 = y0 as i64;
    //     let x1 = x1 as i64;
    //     let y1 = y1 as i64;
    //     let dx = (x1 - x0).abs();
    //     let sx: i64 = if x0 < x1 { 1 } else { -1 };
    //     let dy = -(y1 - y0).abs();
    //     let sy: i64 = if y0 < y1 { 1 } else { -1 };
    //     let mut err = dx + dy;
    //     while x != x1 || y != y1 {
    //         fb[(y as usize * WIDTH + x as usize)..(y as usize * WIDTH + (x as usize + 1))]
    //             .fill(col);
    //         let e2 = 2 * err;
    //         if dy <= e2 {
    //             err += dy;
    //             x += sx;
    //         }
    //         if e2 <= dx {
    //             err += dx;
    //             y += sy;
    //         }
    //     }
    // }

    // fn triangle(
    //     &mut self,
    //     (x0, y0): (usize, usize),
    //     (x1, y1): (usize, usize),
    //     (x2, y2): (usize, usize),
    //     col: Color,
    // ) {
    //     self.line((x0, y0), (x1, y1), col);
    //     self.line((x0, y0), (x2, y2), col);
    //     self.line((x1, y1), (x2, y2), col);
    // }

    // #[allow(dead_code)]
    // fn draw_filled_rect(&mut self, rect: &mut Rect, c: Color) {
    //     let fb = &mut self.buffer;
    //     let Rect { x0, y0, w, h } = *rect;
    //     let y1 = y0 + h as i32;
    //     let x1 = x0 + w as i32;

    //     assert!(y0 <= y1);
    //     assert!(y1 <= HEIGHT as i32);
    //     assert!(x0 <= x1);
    //     assert!(x1 <= WIDTH as i32);

    //     for row in (y0 as usize)..(y1 as usize) {
    //         fb[row * WIDTH + x0 as usize..(row * WIDTH + x1 as usize)].fill(c);
    //     }
    // }

    // #[allow(dead_code)]
    // fn draw_filled_outline_rect(&mut self, rect: &mut Rect, c: Color) {
    //     let fb = &mut self.buffer;
    //     let Rect { x0, y0, w, h } = *rect;
    //     let y1 = y0 + h as i32;
    //     let x1 = x0 + w as i32;

    //     assert!(y0 <= y1);
    //     assert!(y1 <= HEIGHT as i32);
    //     assert!(x0 <= x1);
    //     assert!(x1 <= WIDTH as i32);
    //     fb[(y0 as usize) * WIDTH + x0 as usize..((y0 as usize) * WIDTH + x1 as usize)].fill(c);
    //     for row in ((y0 as usize) + 1)..((y1 as usize) + 1) {
    //         fb[row * WIDTH + x0 as usize..(row * WIDTH + (x0 + 2) as usize)].fill(c);
    //         fb[row * WIDTH + (x1 - 2) as usize..(row * WIDTH + x1 as usize)].fill(c);
    //     }
    //     fb[(y1 as usize) * WIDTH + x0 as usize..((y1 as usize) * WIDTH + x1 as usize)].fill(c);
    // }

    // fn rectangle_bounce<'a>(
    //     &'a mut self,
    //     rect: &'a mut Rect,
    //     c: Color,
    //     x_dir: &'a mut i32,
    //     y_dir: &'a mut i32,
    // ) -> (&'a mut i32, &'a mut i32) {
    //     let y_change = 4;
    //     let x_change = 4;

    //     let y1 = rect.y0 + rect.h as i32;
    //     let x1 = rect.x0 + rect.w as i32;

    //     let mut change = false;

    //     if HEIGHT as i32 - y1 < y_change && *y_dir == 1 {
    //         rect.y0 = (HEIGHT as u32 - rect.h) as i32;
    //         *y_dir = 0;
    //         change = true;
    //     } else if rect.y0 - 0 < y_change && *y_dir == 0 {
    //         rect.y0 = 0;
    //         *y_dir = 1;
    //         change = true;
    //     }

    //     if WIDTH as i32 - x1 < x_change && *x_dir == 1 {
    //         rect.x0 = rect.x0 + (WIDTH as i32 - x1);
    //         *x_dir = 0;
    //         change = true;
    //     } else if rect.x0 - 0 < x_change && *x_dir == 0 {
    //         rect.x0 = 0;
    //         *x_dir = 1;
    //         change = true;
    //     }

    //     if change == false {
    //         if *y_dir == 0 {
    //             rect.y0 = rect.y0 - y_change;
    //         } else if *y_dir == 1 {
    //             rect.y0 = rect.y0 + y_change;
    //         }

    //         if *x_dir == 0 {
    //             rect.x0 = rect.x0 - x_change;
    //         } else if *x_dir == 1 {
    //             rect.x0 = rect.x0 + x_change;
    //         }
    //     }

    //     self.draw_filled_rect(rect, c);

    //     (x_dir, y_dir)
    // }
}
