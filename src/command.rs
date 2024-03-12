use crate::interface::DisplayInterface;

const MAX_GATES: u16 = 296;
const MAX_DUMMY_LINE_PERIOD: u8 = 127;

trait Contains<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool;
}

/// The address increment orientation when writing image data. This configures how the controller
/// will auto-increment the row and column addresses when image data is written using the
/// `WriteImageData` command.
#[derive(Clone, Copy)]
pub enum IncrementAxis {
    /// X direction
    Horizontal,
    /// Y direction
    Vertical,
}

#[derive(Clone, Copy)]
pub enum DataEntryMode {
    DecrementXDecrementY,
    IncrementXDecrementY,
    DecrementXIncrementY,
    IncrementYIncrementX, // POR
}

#[derive(Clone, Copy)]
pub enum TemperatureSensor {
    Internal,
    External,
}

#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum DisplayUpdateSequenceOption {
    EnableClockSignal,
    DisableClockSignal,
    EnableClockSignal_EnableAnalog,
    DisableAnalog_DisableClockSignal,
    EnableClockSignal_LoadLutMode1_DisableClockSignal,
    EnableClockSignal_LoadLutMode2_DisableClockSignal,
    EnableClockSignal_LoadTemp_LoadLutMode1_DisableClockSignal,
    EnableClockSignal_LoadTemp_LoadLutMode2_DisableClockSignal,
    EnableClockSignal_EnableAnalog_DisplayMode1_DisableAnalog_DisableOscillator,
    EnableClockSignal_EnableAnalog_DisplayMode2_DisableAnalog_DisableOscillator,
    EnableClockSignal_LoadTemp_EnableAnalog_DisplayMode1_DisableAnalog_DisableOscillator,
    EnableClockSignal_LoadTemp_EnableAnalog_DisplayMode2_DisableAnalog_DisableOscillator,
}

#[derive(Clone, Copy)]
pub enum RamOption {
    Normal,
    Bypass,
    Invert,
}

#[derive(Clone, Copy)]
pub enum SourceOption {
    SourceFromS0ToS175,
    SourceFromS8ToS167,
}

#[derive(Clone, Copy)]
pub enum DeepSleepMode {
    /// Not sleeping
    Normal,
    /// Deep sleep with RAM preserved
    PreserveRAM,
    /// Deep sleep RAM not preserved
    DiscardRAM,
}

/// A command that can be issued to the controller.
#[derive(Clone, Copy)]
pub enum Command {
    /// Set the MUX of gate lines, scanning sequence and direction
    /// 0: MAX gate lines
    /// 1: Gate scanning sequence and direction
    DriverOutputControl(u16, u8),
    /// Set the gate driving voltage.
    GateDrivingVoltage(u8),
    /// Set the source driving voltage.
    /// 0: VSH1
    /// 1: VSH2
    /// 2: VSL
    SourceDrivingVoltage(u8, u8, u8),
    /// Booster enable with phases 1 to 3 for soft start current and duration setting
    /// 0: Soft start setting for phase 1
    /// 1: Soft start setting for phase 2
    /// 2: Soft start setting for phase 3
    /// 3: Duration setting
    BoosterEnable(u8, u8, u8, u8),
    /// Set the scanning start position of the gate driver
    GateScanStartPosition(u16),
    /// Set deep sleep mode
    DeepSleepMode(DeepSleepMode),
    /// Set the data entry mode and increment axis
    DataEntryMode(DataEntryMode, IncrementAxis),
    /// Perform a soft reset, and reset all parameters to their default values
    /// BUSY will be high when in progress.
    SoftReset,
    // /// Start HV ready detection. Read result with `ReadStatusBit` command
    // StartHVReadyDetection,
    // /// Start VCI level detection
    // /// 0: threshold
    // /// Read result with `ReadStatusBit` command
    // StartVCILevelDetection(u8),
    /// Specify internal or external temperature sensor
    TemperatureSensorSelection(TemperatureSensor),
    /// Write to the temperature sensor register
    WriteTemperatureSensor(u16),
    /// Read from the temperature sensor register
    ReadTemperatureSensor(u16),
    /// Write a command to the external temperature sensor
    WriteExternalTemperatureSensor(u8, u8, u8),
    /// Activate display update sequence. BUSY will be high when in progress.
    UpdateDisplay,
    /// Set RAM content options for update display command.
    /// 0: Black/White RAM option
    /// 1: Red RAM option
    /// 2: Source option
    UpdateDisplayOption1(RamOption, RamOption, SourceOption),
    /// Set display update sequence options
    UpdateDisplayOption2(DisplayUpdateSequenceOption),
    // Read from RAM (not implemented)
    // ReadData,
    /// Enter VCOM sensing and hold for duration defined by VCOMSenseDuration
    /// BUSY will be high when in progress.
    EnterVCOMSensing,
    /// Set VCOM sensing duration
    VCOMSenseDuration(u8),
    // /// Program VCOM register into OTP
    // ProgramVCOMIntoOTP,
    /// Write VCOM register from MCU interface
    WriteVCOM(u8),
    // ReadDisplayOption,
    // ReadUserId,
    // StatusBitRead,
    // ProgramWaveformSetting,
    // LoadWaveformSetting,
    // CalculateCRC,
    // ReadCRC,
    // ProgramOTP,
    // WriteDisplayOption,
    // WriteUserId,
    // OTPProgramMode,
    /// Set the number of dummy line period in terms of gate line width (TGate)
    DummyLinePeriod(u8),
    /// Set the gate line width (TGate)
    GateLineWidth(u8),
    /// Select border waveform for VBD
    BorderWaveform(u8),
    // ReadRamOption,
    /// Set the start/end positions of the window address in the X direction
    /// 0: Start
    /// 1: End
    StartEndXPosition(u8, u8),
    /// Set the start/end positions of the window address in the Y direction
    /// 0: Start
    /// 1: End
    StartEndYPosition(u16, u16),
    /// Auto write red RAM for regular pattern
    AutoWriteRedPattern(u8),
    /// Auto write red RAM for regular pattern
    AutoWriteBlackPattern(u8),
    /// Set RAM X address
    XAddress(u8),
    /// Set RAM Y address
    YAddress(u16),
    /// Set analog block control
    AnalogBlockControl(u8),
    /// Set digital block control
    DigitalBlockControl(u8),
    // Used to terminate frame memory reads
    // Nop,
}

/// Enumerates commands that can be sent to the controller that accept a slice argument buffer. This
/// is separated from `Command` so that the lifetime parameter of the argument buffer slice does
/// not pervade code which never invokes these two commands.
pub enum BufCommand<'buf> {
    /// Write to black/white RAM
    /// 1 = White
    /// 0 = Black
    WriteBlackData(&'buf [u8]),
    /// Write to red RAM
    /// 1 = Red
    /// 0 = Use contents of black/white RAM
    WriteRedData(&'buf [u8]),
    /// Write LUT register (70 bytes)
    WriteLUT(&'buf [u8]),
}

/// Populates data buffer (array) and returns a pair (tuple) with command and
/// appropriately sized slice into populated buffer.
/// E.g.
///
/// let mut buf = [0u8; 4];
/// let (command, data) = pack!(buf, 0x3C, [0x12, 0x34]);
macro_rules! pack {
    ($buf:ident, $cmd:expr,[]) => {
        ($cmd, &$buf[..0])
    };
    ($buf:ident, $cmd:expr,[$arg0:expr]) => {{
        $buf[0] = $arg0;
        ($cmd, &$buf[..1])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        ($cmd, &$buf[..2])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        ($cmd, &$buf[..3])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        $buf[3] = $arg3;
        ($cmd, &$buf[..4])
    }};
}

impl Command {
    /// Execute the command, transmitting any associated data as well.
    pub async fn execute<I: DisplayInterface>(&self, interface: &mut I) -> Result<(), I::Error> {
        use self::Command::*;

        let mut buf = [0u8; 4];
        let (command, data) = match *self {
            DriverOutputControl(gate_lines, scanning_seq_and_dir) => {
                let [upper, lower] = gate_lines.to_be_bytes();
                pack!(buf, 0x01, [lower, upper, scanning_seq_and_dir])
            }
            GateDrivingVoltage(voltages) => pack!(buf, 0x03, [voltages]),
            SourceDrivingVoltage(vsh1, vsh2, vsl) => pack!(buf, 0x04, [vsh1, vsh2, vsl]),
            BoosterEnable(phase1, phase2, phase3, duration) => {
                pack!(buf, 0x0C, [phase1, phase2, phase3, duration])
            }
            GateScanStartPosition(position) => {
                debug_assert!(Contains::contains(&(0..MAX_GATES), position));
                let [upper, lower] = position.to_be_bytes();
                pack!(buf, 0x0F, [lower, upper])
            }
            DeepSleepMode(mode) => {
                let mode = match mode {
                    self::DeepSleepMode::Normal => 0b00,
                    self::DeepSleepMode::PreserveRAM => 0b01,
                    self::DeepSleepMode::DiscardRAM => 0b11,
                };

                pack!(buf, 0x10, [mode])
            }
            DataEntryMode(data_entry_mode, increment_axis) => {
                let mode = match data_entry_mode {
                    self::DataEntryMode::DecrementXDecrementY => 0b00,
                    self::DataEntryMode::IncrementXDecrementY => 0b01,
                    self::DataEntryMode::DecrementXIncrementY => 0b10,
                    self::DataEntryMode::IncrementYIncrementX => 0b11,
                };
                let axis = match increment_axis {
                    IncrementAxis::Horizontal => 0b000,
                    IncrementAxis::Vertical => 0b100,
                };

                pack!(buf, 0x11, [axis | mode])
            }
            SoftReset => pack!(buf, 0x12, []),
            TemperatureSensorSelection(temperature_sensor) => {
                let sensor = match temperature_sensor {
                    TemperatureSensor::External => 0x48_u8,
                    TemperatureSensor::Internal => 0x80_u8,
                };

                pack!(buf, 0x18, [sensor])
            }
            WriteTemperatureSensor(value) => {
                let values = value.to_be_bytes();
                pack!(buf, 0x1A, [values[0], values[1]])
            }
            // ReadTemperatureSensor(u16) => {
            // }
            // WriteExternalTemperatureSensor(u8, u8, u8) => {
            // }
            UpdateDisplay => pack!(buf, 0x20, []),
            UpdateDisplayOption1(black_ram_option, red_ram_option, source_option) => {
                let black = match black_ram_option {
                    RamOption::Normal => 0b0000_0000,
                    RamOption::Bypass => 0b0100_0000,
                    RamOption::Invert => 0b1000_0000,
                };
                let red = match red_ram_option {
                    RamOption::Normal => 0b0000_0000,
                    RamOption::Bypass => 0b0000_0100,
                    RamOption::Invert => 0b0000_1000,
                };
                let source = match source_option {
                    SourceOption::SourceFromS0ToS175 => 0b0000_0000,
                    SourceOption::SourceFromS8ToS167 => 0b1000_0000,
                };
                pack!(buf, 0x21, [black | red, source])
            }
            UpdateDisplayOption2(update_sequence_option) => {
                let option = match update_sequence_option {
                    DisplayUpdateSequenceOption::EnableClockSignal => 0x80_u8,
                    DisplayUpdateSequenceOption::DisableClockSignal => 0x01_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_EnableAnalog => 0xC0_u8,
                    DisplayUpdateSequenceOption::DisableAnalog_DisableClockSignal => 0x03_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_LoadLutMode1_DisableClockSignal => 0x91_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_LoadLutMode2_DisableClockSignal => 0x99_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_LoadTemp_LoadLutMode1_DisableClockSignal => 0xB1_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_LoadTemp_LoadLutMode2_DisableClockSignal => 0xB9_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_EnableAnalog_DisplayMode1_DisableAnalog_DisableOscillator => 0xC7_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_EnableAnalog_DisplayMode2_DisableAnalog_DisableOscillator => 0xCF_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_LoadTemp_EnableAnalog_DisplayMode1_DisableAnalog_DisableOscillator => 0xF7_u8,
                    DisplayUpdateSequenceOption::EnableClockSignal_LoadTemp_EnableAnalog_DisplayMode2_DisableAnalog_DisableOscillator => 0xFF_u8,
                };
                pack!(buf, 0x22, [option])
            }
            // EnterVCOMSensing => {
            // }
            // VCOMSenseDuration(u8) => {
            // }
            WriteVCOM(value) => pack!(buf, 0x2C, [value]),
            DummyLinePeriod(period) => {
                debug_assert!(Contains::contains(&(0..=MAX_DUMMY_LINE_PERIOD), period));
                pack!(buf, 0x3A, [period])
            }
            GateLineWidth(tgate) => pack!(buf, 0x3B, [tgate]),
            BorderWaveform(border_waveform) => pack!(buf, 0x3C, [border_waveform]),
            StartEndXPosition(start, end) => pack!(buf, 0x44, [start, end]),
            StartEndYPosition(start, end) => {
                let [start_upper, start_lower] = start.to_be_bytes();
                let [end_upper, end_lower] = end.to_be_bytes();
                pack!(buf, 0x45, [start_lower, start_upper, end_lower, end_upper])
            }
            // AutoWriteRedPattern(u8) => {
            // }
            // AutoWriteBlackPattern(u8) => {
            // }
            XAddress(address) => pack!(buf, 0x4E, [address]),
            YAddress(address) => {
                let [upper, lower] = address.to_be_bytes();
                pack!(buf, 0x4F, [lower, upper])
            }
            AnalogBlockControl(value) => pack!(buf, 0x74, [value]),
            DigitalBlockControl(value) => pack!(buf, 0x7E, [value]),
            _ => unimplemented!(),
        };

        interface.send_command(command).await?;
        if data.is_empty() {
            Ok(())
        } else {
            interface.send_data(data).await
        }
    }
}

impl<'buf> BufCommand<'buf> {
    /// Execute the command, transmitting the associated buffer as well.
    pub async fn execute<I: DisplayInterface>(&self, interface: &mut I) -> Result<(), I::Error> {
        use self::BufCommand::*;

        let (command, data) = match self {
            WriteBlackData(buffer) => (0x24, buffer),
            WriteRedData(buffer) => (0x26, buffer),
            WriteLUT(buffer) => (0x32, buffer),
        };

        interface.send_command(command).await?;
        if data.is_empty() {
            Ok(())
        } else {
            interface.send_data(data).await
        }
    }
}

impl<C> Contains<C> for core::ops::Range<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool {
        item >= self.start && item < self.end
    }
}

impl<C> Contains<C> for core::ops::RangeInclusive<C>
where
    C: Copy + PartialOrd,
{
    fn contains(&self, item: C) -> bool {
        item >= *self.start() && item <= *self.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::delay::DelayNs;

    struct MockInterface {
        data: [u8; 256],
        offset: usize,
    }

    impl MockInterface {
        fn new() -> Self {
            MockInterface {
                data: [0; 256],
                offset: 0,
            }
        }

        fn write(&mut self, byte: u8) {
            self.data[self.offset] = byte;
            self.offset += 1;
        }

        fn data(&self) -> &[u8] {
            &self.data[0..self.offset]
        }
    }

    impl DisplayInterface for MockInterface {
        type Error = ();

        /// Send a command to the controller.
        ///
        /// Prefer calling `execute` on a [Command](../command/enum.Command.html) over calling this
        /// directly.
        async fn send_command(&mut self, command: u8) -> Result<(), Self::Error> {
            self.write(command);
            Ok(())
        }

        /// Send data for a command.
        async fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            for byte in data {
                self.write(*byte)
            }
            Ok(())
        }

        /// Reset the controller.
        async fn reset<D: DelayNs>(&mut self, _delay: &mut D) {
            self.data = [0; 256];
            self.offset = 0;
        }

        /// Wait for the controller to indicate it is not busy.
        async fn busy_wait(&mut self) {
            // nop
        }
    }

    #[futures_test::test]
    async fn test_command_execute() {
        let mut interface = MockInterface::new();
        let upper = 0x12;
        let lower = 0x34;
        let scanning_seq_and_dir = 1;
        let command = Command::DriverOutputControl(0x1234, scanning_seq_and_dir);

        command.execute(&mut interface).await.unwrap();
        assert_eq!(
            interface.data(),
            &[0x01, lower, upper, scanning_seq_and_dir]
        );
    }
}
