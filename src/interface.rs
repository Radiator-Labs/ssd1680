use core::{fmt::Debug, future::Future};
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embassy_time::Timer;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::spi::SpiDevice;

// Section 15.2 of the HINK-E0213A07 data sheet says to hold for 10ms
const RESET_DELAY_MS: u64 = 10;
const TIMEOUT_MS: u32 = 5_000;
const NUM_RESET_DELAYS_IS_TIMEOUT: u32 = TIMEOUT_MS / (RESET_DELAY_MS as u32);

/// Trait implemented by displays to provide implementation of core functionality.
pub trait DisplayInterface {
    type Error;

    /// Send a command to the controller.
    ///
    /// Prefer calling `execute` on a [Command](../command/enum.Command.html) over calling this
    /// directly.
    fn send_command(&mut self, command: u8) -> impl Future<Output = Result<(), Self::Error>>;

    /// Send data for a command.
    fn send_data(&mut self, data: &[u8]) -> impl Future<Output = Result<(), Self::Error>>;

    /// Reset the controller.
    fn reset(&mut self) -> impl Future<Output = ()>;

    /// Wait for the controller to indicate it is not busy.
    fn busy_wait(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// The hardware interface to a display.
///
/// ### Example
///
/// This example uses the Linux implementation of the embedded HAL traits to build a display
/// interface. For a complete example see [the Raspberry Pi Inky pHAT example](https://github.com/Radiator-Labs/ssd1680/blob/master/examples/raspberry_pi_inky_phat.rs).
///
/// ```ignore
/// extern crate linux_embedded_hal;
/// use linux_embedded_hal::spidev::{self, SpidevOptions};
/// use linux_embedded_hal::sysfs_gpio::Direction;
/// use linux_embedded_hal::Delay;
/// use linux_embedded_hal::{Pin, Spidev};
///
/// extern crate ssd1680;
/// use ssd1680::{Builder, Dimensions, Display, GraphicDisplay, Rotation};
///
/// // Configure SPI
/// let mut spi = Spidev::open("/dev/spidev0.0").expect("SPI device");
/// let options = SpidevOptions::new()
///     .bits_per_word(8)
///     .max_speed_hz(4_000_000)
///     .mode(spidev::SPI_MODE_0)
///     .build();
/// spi.configure(&options).expect("SPI configuration");
///
/// // https://pinout.xyz/pinout/inky_phat
/// // Configure Digital I/O Pins
/// let cs = Pin::new(8); // BCM8
/// cs.export().expect("cs export");
/// while !cs.is_exported() {}
/// cs.set_direction(Direction::Out).expect("CS Direction");
/// cs.set_value(1).expect("CS Value set to 1");
///
/// let busy = Pin::new(17); // BCM17
/// busy.export().expect("busy export");
/// while !busy.is_exported() {}
/// busy.set_direction(Direction::In).expect("busy Direction");
///
/// let dc = Pin::new(22); // BCM22
/// dc.export().expect("dc export");
/// while !dc.is_exported() {}
/// dc.set_direction(Direction::Out).expect("dc Direction");
/// dc.set_value(1).expect("dc Value set to 1");
///
/// let reset = Pin::new(27); // BCM27
/// reset.export().expect("reset export");
/// while !reset.is_exported() {}
/// reset
///     .set_direction(Direction::Out)
///     .expect("reset Direction");
/// reset.set_value(1).expect("reset Value set to 1");
///
/// // Build the interface from the pins and SPI device
/// let controller = ssd1680::Interface::new(spi, cs, busy, dc, reset);

#[allow(dead_code)] // Prevent warning about CS being unused
pub struct Interface<SpiDev, BUS, CS, BUSY, DC, RESET>
where
    SpiDev: SpiDevice<u8, Error = SpiDeviceError<BUS, CS>>,
    BUS: embedded_hal::spi::Error + Debug + PartialEq,
    CS: Debug + PartialEq,
{
    /// SPI Device interface (chip select is owned by this)
    spi: SpiDev,
    /// Active low busy pin (input)
    busy: BUSY,
    /// Data/Command Control Pin (High for data, Low for command) (output)
    dc: DC,
    /// Pin for resetting the controller (output)
    reset: RESET,
}

impl<SpiDev, BUS, CS, BUSY, DC, RESET> Interface<SpiDev, BUS, CS, BUSY, DC, RESET>
where
    SpiDev: SpiDevice<u8, Error = SpiDeviceError<BUS, CS>>,
    BUS: embedded_hal::spi::Error + Debug + PartialEq,
    CS: Debug + PartialEq,
    BUSY: InputPin,
    DC: OutputPin,
    RESET: OutputPin,
{
    /// Create a new Interface from embedded hal traits.
    pub fn new(spi: SpiDev, busy: BUSY, dc: DC, reset: RESET) -> Self {
        Self {
            spi,
            busy,
            dc,
            reset,
        }
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), SpiDeviceError<BUS, CS>> {
        // Linux has a default limit of 4096 bytes per SPI transfer
        // https://github.com/torvalds/linux/blob/ccda4af0f4b92f7b4c308d3acc262f4a7e3affad/drivers/spi/spidev.c#L93
        if cfg!(target_os = "linux") {
            for data_chunk in data.chunks(4096) {
                self.spi.write(data_chunk).await?;
            }
        } else {
            self.spi.write(data).await?;
        }

        Ok(())
    }

    async fn busy_wait_with_timeout(&mut self) -> Result<(), ()> {
        let mut count = 0;
        while match self.busy.is_high() {
            Ok(x) => {
                if x {
                    Timer::after_millis(RESET_DELAY_MS).await;
                }
                x
            }
            _ => return Err(()),
        } {
            if count > NUM_RESET_DELAYS_IS_TIMEOUT {
                return Err(());
            }
            count += 1;
        }
        Ok(())
    }
}

impl<SpiDev, BUS, CS, BUSY, DC, RESET> DisplayInterface
    for Interface<SpiDev, BUS, CS, BUSY, DC, RESET>
where
    SpiDev: SpiDevice<u8, Error = SpiDeviceError<BUS, CS>>,
    BUS: embedded_hal::spi::Error + Debug + PartialEq,
    CS: Debug + PartialEq,
    BUSY: InputPin,
    DC: OutputPin,
    DC::Error: Debug,
    RESET: OutputPin,
    RESET::Error: Debug,
{
    type Error = SpiDev::Error;

    async fn reset(&mut self) {
        self.reset.set_low().unwrap();
        Timer::after_millis(RESET_DELAY_MS).await;
        self.reset.set_high().unwrap();
        Timer::after_millis(RESET_DELAY_MS).await;
    }

    async fn send_command(&mut self, command: u8) -> Result<(), SpiDeviceError<BUS, CS>> {
        self.dc.set_low().unwrap();
        self.write(&[command]).await?;
        self.dc.set_high().unwrap();

        Ok(())
    }

    async fn send_data(&mut self, data: &[u8]) -> Result<(), SpiDeviceError<BUS, CS>> {
        self.dc.set_high().unwrap();
        self.write(data).await
    }

    async fn busy_wait(&mut self) -> Result<(), SpiDeviceError<BUS, CS>> {
        if self.busy_wait_with_timeout().await.is_err() {
            Err(SpiDeviceError::Config)
        } else {
            Ok(())
        }
    }
}
