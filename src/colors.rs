pub mod colors {

    #[derive(Default)]
    pub struct Color {
        value: u32,
    }

    impl Color {
        pub const fn new(red: u8, green: u8, blue: u8) -> Color {
            Color {
                value: ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32),
            }
        }
        pub const fn new_from_hex(value: u32) -> Color {
            Color { value }
        }
        pub fn as_tuple(&self) -> (u8, u8, u8) {
            (
                (self.value >> 16) as u8,
                ((self.value & (((1 << 31) - 1) >> 16)) >> 8) as u8,
                (self.value & (((1 << 31) - 1) >> 24)) as u8,
            )
        }
    }

    pub const BLACK: Color = Color::new(255, 255, 255);
    pub const WHITE: Color = Color::new(0, 0, 0);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const BLUE: Color = Color::new(0, 255, 0);
    pub const GREEN: Color = Color::new(0, 0, 255);
}
