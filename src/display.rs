use crate::{
    command::{
        BufCommand, Command, DataEntryMode, DeepSleepMode, DisplayUpdateSequenceOption,
        IncrementAxis, RamOption, SourceOption, TemperatureSensor,
    },
    config::Config,
    interface::DisplayInterface,
};
use embedded_hal::delay::DelayNs;

// Max display resolution is 176x296 // was 160x296
/// The maximum number of rows supported by the controller
pub const MAX_GATE_OUTPUTS: u16 = 296;
/// The maximum number of columns supported by the controller
pub const MAX_SOURCE_OUTPUTS: u8 = 176;

// Magic numbers from the data sheet
// const ANALOG_BLOCK_CONTROL_MAGIC: u8 = 0x54;
// const DIGITAL_BLOCK_CONTROL_MAGIC: u8 = 0x3B;

/// Represents the dimensions of the display.
pub struct Dimensions {
    /// The number of rows the display has.
    ///
    /// Must be less than or equal to MAX_GATE_OUTPUTS.
    pub rows: u16,
    /// The number of columns the display has.
    ///
    /// Must be less than or equal to MAX_SOURCE_OUTPUTS.
    pub cols: u8,
}

/// Represents the physical rotation of the display relative to the native orientation.
///
/// For example the native orientation of the Inky pHAT display is a tall (portrait) 104x212
/// display. `Rotate270` can be used to make it the right way up when attached to a Raspberry Pi
/// Zero with the ports on the top.
#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Default for Rotation {
    /// Default is no rotation (`Rotate0`).
    fn default() -> Self {
        Rotation::Rotate0
    }
}

/// A configured display with a hardware interface.
pub struct Display<'a, I>
where
    I: DisplayInterface,
{
    interface: I,
    config: Config<'a>,
}

impl<'a, I> Display<'a, I>
where
    I: DisplayInterface,
{
    /// Create a new display instance from a DisplayInterface and Config.
    ///
    /// The `Config` is typically created with `config::Builder`.
    pub fn new(interface: I, config: Config<'a>) -> Self {
        Self { interface, config }
    }

    /// Perform a hardware reset followed by software reset.
    ///
    /// This will wake a controller that has previously entered deep sleep.
    pub async fn reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.chip_reset(delay).await?;
        self.init_for_fast().await?;
        self.init().await
    }

    async fn chip_reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.interface.reset(delay).await;
        self.interface.busy_wait().await;
        Command::SoftReset.execute(&mut self.interface).await
    }

    /// Initialize the controller according to Section 9: Typical Operating Sequence
    /// from the data sheet
    async fn init(&mut self) -> Result<(), I::Error> {
        // Matches Section 9: Typical Operating Sequence from the data sheet
        self.interface.busy_wait().await;
        Command::DriverOutputControl(self.config.dimensions.rows - 1, 0x00)
            .execute(&mut self.interface)
            .await?;
        Command::DataEntryMode(
            DataEntryMode::IncrementYIncrementX, // DataEntryMode::IncrementXDecrementY
            IncrementAxis::Horizontal,
        )
        .execute(&mut self.interface)
        .await?;
        Command::TemperatureSensorSelection(TemperatureSensor::Internal)
            .execute(&mut self.interface)
            .await?;

        let end = self.config.dimensions.cols / 8 - 1;
        Command::StartEndXPosition(0, end)
            .execute(&mut self.interface)
            .await?;
        Command::StartEndYPosition(0, self.config.dimensions.rows - 1)
            .execute(&mut self.interface)
            .await?;

        Command::BorderWaveform(0x05_u8)
            .execute(&mut self.interface)
            .await?;
        Command::UpdateDisplayOption1(
            RamOption::Normal,
            RamOption::Normal,
            SourceOption::SourceFromS8ToS167,
        )
        .execute(&mut self.interface)
        .await?;

        Command::XAddress(0x00).execute(&mut self.interface).await?;
        Command::YAddress(self.config.dimensions.rows - 1)
            .execute(&mut self.interface)
            .await?;

        Ok(())
    }

    async fn init_for_fast(&mut self) -> Result<(), I::Error> {
        // Matches code example from GoodDisplay
        Command::TemperatureSensorSelection(TemperatureSensor::Internal)
            .execute(&mut self.interface)
            .await?;
        Command::UpdateDisplayOption2(
            DisplayUpdateSequenceOption::EnableClockSignal_LoadTemp_LoadLutMode1_DisableClockSignal,
        )
        .execute(&mut self.interface)
        .await?;
        Command::UpdateDisplay.execute(&mut self.interface).await?;
        self.interface.busy_wait().await;

        Command::WriteTemperatureSensor(0x6400)
            .execute(&mut self.interface)
            .await?;

        Command::UpdateDisplayOption2(
            DisplayUpdateSequenceOption::EnableClockSignal_LoadLutMode1_DisableClockSignal,
        )
        .execute(&mut self.interface)
        .await?;
        Command::UpdateDisplay.execute(&mut self.interface).await?;
        self.interface.busy_wait().await;

        Ok(())
    }

    /// Update the display by writing the supplied B/W and Red buffers to the controller.
    ///
    /// This method will write the two buffers to the controller then initiate the update
    /// display command. Currently it will busy wait until the update has completed.
    pub async fn update<D: DelayNs>(
        &mut self,
        black: &[u8],
        red: &[u8],
        delay: &mut D,
    ) -> Result<(), I::Error> {
        self.update_impl(black, red, delay).await?;

        // Kick off the display update
        Command::UpdateDisplayOption2(DisplayUpdateSequenceOption::EnableClockSignal_EnableAnalog_DisplayMode1_DisableAnalog_DisableOscillator).execute(&mut self.interface).await?; // was 0xC7, should be 0xCF
        Command::UpdateDisplay.execute(&mut self.interface).await?;

        Ok(())
    }

    async fn update_impl<D: DelayNs>(
        &mut self,
        black: &[u8],
        red: &[u8],
        _delay: &mut D,
    ) -> Result<(), I::Error> {
        self.interface.busy_wait().await;
        // Write the B/W RAM
        let buf_size = self.rows() as usize * self.cols() as usize;
        let limit_adder = if buf_size % 8 != 0 { 1 } else { 0 };
        let buf_limit = (buf_size / 8) + limit_adder;

        Command::XAddress(0).execute(&mut self.interface).await?;
        Command::YAddress(self.config.dimensions.rows - 1)
            .execute(&mut self.interface)
            .await?;
        BufCommand::WriteBlackData(&black[..buf_limit])
            .execute(&mut self.interface)
            .await?;

        // Write the Red RAM
        Command::XAddress(0).execute(&mut self.interface).await?;
        Command::YAddress(self.config.dimensions.rows - 1)
            .execute(&mut self.interface)
            .await?;
        BufCommand::WriteRedData(&red[..buf_limit])
            .execute(&mut self.interface)
            .await?;

        Ok(())
    }

    /// Enter deep sleep mode.
    ///
    /// This puts the display controller into a low power mode. `reset` must be called to wake it
    /// from sleep.
    pub async fn deep_sleep(&mut self) -> Result<(), I::Error> {
        self.interface.busy_wait().await;
        Command::DeepSleepMode(DeepSleepMode::PreserveRAM)
            .execute(&mut self.interface)
            .await
    }

    /// Returns the number of rows the display has.
    pub fn rows(&self) -> u16 {
        self.config.dimensions.rows
    }

    /// Returns the number of columns the display has.
    pub fn cols(&self) -> u8 {
        self.config.dimensions.cols
    }

    /// Returns the rotation the display was configured with.
    pub fn rotation(&self) -> Rotation {
        self.config.rotation
    }
}
