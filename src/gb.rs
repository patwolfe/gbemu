pub use self::dimensions::PIXELS_TOTAL as total_pixels;
pub use self::dimensions::PIXELS_X as screen_width;
pub use self::dimensions::PIXELS_Y as screen_height;
pub use self::init_state::INIT_PC as init_pc_value;
pub use self::init_state::INIT_SP as init_sp_value;
pub use self::interrupt_pointers::IE as ie;
pub use self::interrupt_pointers::IE as iflags;
pub use self::mmio_pointers::DMA_TRANSFER as dma_reg;
pub use self::mmio_pointers::LCDC as lcdc_addr;
pub use self::mmio_pointers::LCD_STATUS as lcd_stat;
pub use self::mmio_pointers::LY as ly_addr;
pub use self::mmio_pointers::LYC as lyc_addr;
pub use self::mmio_pointers::OBJ_PALETTE_0 as obp0_addr;
pub use self::mmio_pointers::OBJ_PALETTE_1 as obp1_addr;
pub use self::mmio_pointers::SCX as scx_addr;
pub use self::mmio_pointers::SCY as scy_addr;
pub use self::mmio_pointers::WX as wx_addr;
pub use self::mmio_pointers::WY as wy_addr;
pub use self::timings::CYCLES_PER_FRAME as cycles_per_frame;

pub mod dimensions {
    pub const PIXELS_Y: usize = 144;
    pub const PIXELS_X: usize = 160;
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
    pub const SCX: u16 = 0xFF43;
    pub const LY: u16 = 0xFF44;
    pub const LYC: u16 = 0xFF45;
    pub const DMA_TRANSFER: u16 = 0xFF46;
    pub const BG_PALETTE: u16 = 0xFF47;
    pub const OBJ_PALETTE_0: u16 = 0xFF48;
    pub const OBJ_PALETTE_1: u16 = 0xFF49;
    pub const WY: u16 = 0xFF4A;
    pub const WX: u16 = 0xFF4B;
}

pub mod interrupt_pointers {
    pub const IE: u16 = 0xFFFF;
    pub const IF: u16 = 0xFF0F;
}

pub mod timings {
    pub const CYCLES_PER_FRAME: u32 = 69905;
}
