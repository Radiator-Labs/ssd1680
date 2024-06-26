/// Represents the state of a pixel in the display
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::pixelcolor::raw::RawU8;
#[cfg(feature = "graphics")]
use self::embedded_graphics::prelude::*;
#[cfg(feature = "graphics")]
impl PixelColor for Color {
    type Raw = RawU8;
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 => Color::Black,
            1 => Color::White,
            _ => panic!("invalid color value"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8() {
        assert_eq!(Color::Black, Color::from(0u8));
        assert_eq!(Color::White, Color::from(1u8));
    }

    #[test]
    fn from_u8_panic() {
        for val in 2..=u8::MAX {
            extern crate std;
            let result = std::panic::catch_unwind(|| Color::from(val));
            assert!(result.is_err());
        }
    }
}
