use crate::{
    color::Color,
    display::{Display, Rotation},
    interface::DisplayInterface,
};
use core::{
    convert::{AsMut, AsRef},
    ops::{Deref, DerefMut},
};
use embedded_hal::delay::DelayNs;

/// A display that holds buffers for drawing into and updating the display from.
///
/// When the `graphics` feature is enabled `GraphicDisplay` implements the `Draw` trait from
/// [embedded-graphics](https://crates.io/crates/embedded-graphics). This allows basic shapes and
/// text to be drawn on the display.
pub struct GraphicDisplay<'a, I, B = &'a mut [u8]>
where
    I: DisplayInterface,
{
    display: Display<'a, I>,
    black_buffer: B,
    work_buffer: B,
}

impl<'a, I, B> GraphicDisplay<'a, I, B>
where
    I: DisplayInterface,
    B: AsMut<[u8]>,
    B: AsRef<[u8]>,
{
    /// Promote a `Display` to a `GraphicDisplay`.
    ///
    /// B/W buffer for drawing into must be supplied. These should be `rows` * `cols` in
    /// length.
    pub fn new(display: Display<'a, I>, black_buffer: B, work_buffer: B) -> Self {
        GraphicDisplay {
            display,
            black_buffer,
            work_buffer,
        }
    }

    /// Update the display by writing the buffers to the controller.
    pub async fn update<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.display.update(self.black_buffer.as_ref(), delay).await
    }

    /// Update the display by writing the buffers to the controller.
    pub async fn partial_update<D: DelayNs>(
        &mut self,
        delay: &mut D,
        start_x_px: u16,
        start_y_px: u16,
        width_px: u16,
        height_px: u16,
    ) -> Result<(), I::Error> {
        let work_buf_ref = self.work_buffer.as_mut();
        let sub_image = make_sub_image(
            self.black_buffer.as_ref(),
            work_buf_ref,
            self.display.cols_as_bytes(),
            start_x_px,
            start_y_px,
            width_px,
            height_px,
        );
        self.display
            .partial_update(
                sub_image, delay, start_x_px, start_y_px, width_px, height_px,
            )
            .await
    }

    /// Clear the buffers, filling them a single color.
    pub fn clear(&mut self, color: Color) {
        let black = match color {
            Color::White => 0xFF,
            Color::Black => 0x00,
        };

        for byte in &mut self.black_buffer.as_mut().iter_mut() {
            *byte = black; // background_color.get_byte_value();
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let (index, bit) = rotation(
            x,
            y,
            self.cols() as u32,
            self.rows() as u32,
            self.rotation(),
        );
        let index = index as usize;

        match color {
            Color::Black => {
                self.black_buffer.as_mut()[index] &= !bit;
            }
            Color::White => {
                self.black_buffer.as_mut()[index] |= bit;
            }
        }
    }
}

impl<'a, I, B> Deref for GraphicDisplay<'a, I, B>
where
    I: DisplayInterface,
{
    type Target = Display<'a, I>;

    fn deref(&self) -> &Display<'a, I> {
        &self.display
    }
}

impl<'a, I, B> DerefMut for GraphicDisplay<'a, I, B>
where
    I: DisplayInterface,
{
    fn deref_mut(&mut self) -> &mut Display<'a, I> {
        &mut self.display
    }
}

fn rotation(x: u32, y: u32, width: u32, height: u32, rotation: Rotation) -> (u32, u8) {
    match rotation {
        Rotation::Rotate0 => (x / 8 + (width / 8) * y, 0x80 >> (x % 8)),
        Rotation::Rotate90 => ((width - 1 - y) / 8 + (width / 8) * x, 0x01 << (y % 8)),
        Rotation::Rotate180 => (
            ((width / 8) * height - 1) - (x / 8 + (width / 8) * y),
            0x01 << (x % 8),
        ),
        Rotation::Rotate270 => (y / 8 + (height - 1 - x) * (width / 8), 0x80 >> (y % 8)),
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics;
#[cfg(feature = "graphics")]
use self::embedded_graphics::prelude::*;

#[cfg(feature = "graphics")]
impl<'a, I, B> DrawTarget for GraphicDisplay<'a, I, B>
where
    I: DisplayInterface,
    B: AsMut<[u8]>,
    B: AsRef<[u8]>,
{
    type Color = Color;
    type Error = core::convert::Infallible;

    fn draw_iter<Iter>(&mut self, pixels: Iter) -> Result<(), Self::Error>
    where
        Iter: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let sz = self.size();
        for Pixel(Point { x, y }, color) in pixels {
            let x = x as u32;
            let y = y as u32;
            if x < sz.width && y < sz.height {
                self.set_pixel(x, y, color)
            }
        }
        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<'a, I, B> OriginDimensions for GraphicDisplay<'a, I, B>
where
    I: DisplayInterface,
{
    fn size(&self) -> Size {
        match self.rotation() {
            Rotation::Rotate0 | Rotation::Rotate180 => {
                Size::new(self.cols().into(), self.rows().into())
            }
            Rotation::Rotate90 | Rotation::Rotate270 => {
                Size::new(self.rows().into(), self.cols().into())
            }
        }
    }
}

#[allow(clippy::indexing_slicing)]
fn make_sub_image<'a>(
    black_buffer: &[u8],
    work_buffer: &'a mut [u8],
    display_width_as_bytes: u8,
    start_x_px: u16,
    start_y_px: u16,
    width_px: u16,
    height_px: u16,
) -> &'a [u8] {
    let mut at = 0_usize;
    let start_x_bytes = start_x_px / 8;
    let width_bytes = width_px / 8;
    let end_y_px = start_y_px + height_px;
    for i in start_y_px..end_y_px {
        let start_x = ((i * display_width_as_bytes as u16) + start_x_bytes) as usize;
        let end_x = start_x + width_bytes as usize;
        for b in black_buffer.iter().take(end_x).skip(start_x) {
            work_buffer[at] = *b;
            at += 1;
        }
    }
    let num_bytes = (width_bytes * height_px) as usize;
    &work_buffer[0..num_bytes]
}

#[cfg(test)]
mod tests {
    use self::embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
    use super::*;
    use crate::{
        color::Color,
        config::Builder,
        display::{Dimensions, Display, Rotation},
        graphics::GraphicDisplay,
    };

    const ROWS: u16 = 3;
    const COLS: u8 = 8;
    const BUFFER_SIZE: usize = (ROWS * COLS as u16) as usize / 8;

    struct MockInterface {}
    struct MockError {}

    impl MockInterface {
        fn new() -> Self {
            MockInterface {}
        }
    }

    impl DisplayInterface for MockInterface {
        type Error = MockError;

        async fn reset<D: DelayNs>(&mut self, _delay: &mut D) {}

        async fn send_command(&mut self, _command: u8) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn send_data(&mut self, _data: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn busy_wait<D: DelayNs>(&mut self, _delay: &mut D) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn build_mock_display<'a>() -> Display<'a, MockInterface> {
        let interface = MockInterface::new();
        let dimensions = Dimensions {
            rows: ROWS,
            cols: COLS,
        };

        let config = Builder::new()
            .dimensions(dimensions)
            .rotation(Rotation::Rotate270)
            .build()
            .expect("invalid config");
        Display::new(interface, config)
    }

    #[test]
    fn clear_white() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut work_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut work_buffer);
            display.clear(Color::White);
        }

        assert_eq!(black_buffer, [0xFF, 0xFF, 0xFF]);
        assert_eq!(work_buffer, [0_u8; BUFFER_SIZE]);
    }

    #[test]
    fn clear_black() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut work_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut work_buffer);
            display.clear(Color::Black);
        }

        assert_eq!(black_buffer, [0x00, 0x00, 0x00]);
        assert_eq!(work_buffer, [0_u8; BUFFER_SIZE]);
    }

    #[test]
    fn draw_rect_white() {
        let mut black_buffer = [0u8; BUFFER_SIZE];
        let mut work_buffer = [0u8; BUFFER_SIZE];

        {
            let mut display =
                GraphicDisplay::new(build_mock_display(), &mut black_buffer, &mut work_buffer);

            Rectangle::with_corners(Point::new(0, 0), Point::new(2, 2))
                .into_styled(
                    PrimitiveStyleBuilder::new()
                        .stroke_color(Color::White)
                        .stroke_width(1)
                        .build(),
                )
                .draw(&mut display)
                .unwrap()
        }

        #[rustfmt::skip]
        assert_eq!(black_buffer, [0b11100000,
                                  0b10100000,
                                  0b11100000]);
        assert_eq!(work_buffer, [0_u8; BUFFER_SIZE]);
    }

    #[test]
    fn make_sub_image_creates_subset_image_with_8_pixels_per_byte_horizontally() {
        const COLS: u16 = 48; // 6 bytes
        const ROWS: u16 = 5;
        const PIXELS_PER_BYTE: u16 = 8;
        const BUFFER_SIZE: usize = ((COLS / PIXELS_PER_BYTE) * ROWS) as usize;
        let buffer: [u8; BUFFER_SIZE] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x21, 0x22,
            0x23, 0x24, 0x25, 0x36, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x41, 0x42, 0x43, 0x44,
            0x45, 0x46,
        ];
        let mut work_buffer = [0_u8; BUFFER_SIZE];
        let start_x_px = 16;
        let start_y_px = 1;
        let width_px = 24;
        let height_px = 2;
        let expected_buffer = [0x13, 0x14, 0x15, 0x23, 0x24, 0x25];
        let expected_size = ((width_px / 8) * 2) as usize;
        let result_slice = make_sub_image(
            &buffer,
            &mut work_buffer,
            (COLS / PIXELS_PER_BYTE) as u8,
            start_x_px,
            start_y_px,
            width_px,
            height_px,
        );
        assert_eq!(result_slice.len(), expected_size);
        assert_eq!(result_slice, expected_buffer);
    }
}
