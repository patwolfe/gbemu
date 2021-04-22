pub struct Memory {
    pub bootrom: Box<[u8]>,
}

impl Memory {
    pub fn initialize() -> Memory {
        let bootrom = Box::new([0; 16]);
        Memory { bootrom }
    }

    pub fn read_byte(&self, ptr: u16) -> u8 {
        self.bootrom[ptr as usize]
    }
}
