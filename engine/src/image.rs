type Color = (u8, u8, u8, u8);

// const WIDTH: usize = 320;
// const HEIGHT: usize = 240;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32, // (x0, y0) is the left upper corner
    pub w: u32,
    pub h: u32, // (x1, y1) is the bottom right corner
}

impl Rect {
    pub fn new(x0: i32, y0: i32, w: u32, h: u32) -> Rect {
        Rect { x0, y0, w, h }
    }

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

    // Maybe add functions to test if a point is inside the rect...
    // Or whether two rects overlap...
}

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

    fn clear(&mut self, c: Color) {
        self.buffer.fill(c);
    }

    fn as_slice(&self) -> &[Color] {
        &self.buffer
    }

    pub fn bitblt(&self) {
        // TODO: write bitblt function
    }

    // THE FOLLOWING CODE IS MY STUFF FROM THE RECTANGLE LAB
    // MIGHT BE USEFUL FOR THE GAME

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
