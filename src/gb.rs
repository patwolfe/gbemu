pub use self::dimensions::PIXELS_TOTAL as total_pixels;
pub use self::dimensions::PIXELS_X as screen_width;
pub use self::dimensions::PIXELS_Y as screen_height;
pub use self::init_state::INIT_PC as init_pc_value;
pub use self::init_state::INIT_SP as init_sp_value;
pub use self::mmio_pointers::LCDC as lcdc_addr;
pub use self::mmio_pointers::LCD_STATUS as lcd_stat;
pub use self::mmio_pointers::LY as ly_addr;
pub use self::mmio_pointers::LYC as lyc_addr;
pub use self::mmio_pointers::SCY as scy_addr;
pub use self::mmio_pointers::WY as wy_addr;

pub mod dimensions {
    pub const PIXELS_X: usize = 144;
    pub const PIXELS_Y: usize = 160;
    pub const PIXELS_TOTAL: usize = PIXELS_X * PIXELS_Y;
}

pub mod init_state {
    pub const INIT_PC: u16 = 0x0100;
    pub const INIT_SP: u16 = 0xFFFE;
}

pub mod mmio_pointers {
    pub const LCDC: u16 = 0xFF40;
    pub const LCD_STATUS: u16 = 0xFF41;
    pub const SCY: u16 = 0xFF42;
    pub const LY: u16 = 0xFF44;
    pub const LYC: u16 = 0xFF45;
    pub const WY: u16 = 0xFF4A;
}
