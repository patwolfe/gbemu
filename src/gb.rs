pub use self::dimensions::PIXELS_TOTAL as total_pixels;
pub use self::dimensions::PIXELS_X as screen_width;
pub use self::dimensions::PIXELS_Y as screen_height;
pub mod dimensions {
    pub const PIXELS_X: usize = 144;
    pub const PIXELS_Y: usize = 160;
    pub const PIXELS_TOTAL: usize = PIXELS_X * PIXELS_Y;
}
