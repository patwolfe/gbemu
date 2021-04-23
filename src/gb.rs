pub use self::dimensions::PIXELS_TOTAL as total_pixels;
pub use self::dimensions::PIXELS_X as screen_width;
pub use self::dimensions::PIXELS_Y as screen_height;
pub use self::init_state::INIT_PC as init_pc_value;
pub use self::init_state::INIT_SP as init_sp_value;

pub mod dimensions {
    pub const PIXELS_X: usize = 144;
    pub const PIXELS_Y: usize = 160;
    pub const PIXELS_TOTAL: usize = PIXELS_X * PIXELS_Y;
}

pub mod init_state {
    pub const INIT_PC: u16 = 0x0100;
    pub const INIT_SP: u16 = 0xFFFE;
}
