use core::fmt::Debug;
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embedded_hal::digital::OutputPin;

#[allow(clippy::exhaustive_enums)]
#[derive(Debug, PartialEq)]
pub enum Ssd1680Error<BUS, CS, DC, RESET>
where
    BUS: embedded_hal::spi::Error + Debug + PartialEq,
    CS: Debug + PartialEq,
    DC: OutputPin,
    DC::Error: Debug,
    RESET: OutputPin,
    RESET::Error: Debug,
{
    SpiError(SpiDeviceError<BUS, CS>),
    DcPinError(DC::Error),
    ResetPinError(RESET::Error),
}

impl<BUS, CS, DC, RESET> From<SpiDeviceError<BUS, CS>> for Ssd1680Error<BUS, CS, DC, RESET>
where
    BUS: embedded_hal::spi::Error + Debug + PartialEq,
    CS: Debug + PartialEq,
    DC: OutputPin,
    DC::Error: Debug,
    RESET: OutputPin,
    RESET::Error: Debug,
{
    fn from(value: SpiDeviceError<BUS, CS>) -> Self {
        Self::SpiError(value)
    }
}
