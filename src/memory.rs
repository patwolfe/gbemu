pub struct Memory {
    pub bootrom: Box<[u8]>,
}

impl Memory {
    pub fn initialize() -> Memory {
        let bootrom = Box::new([0; 16]);
        Memory { bootrom }
    }
}
