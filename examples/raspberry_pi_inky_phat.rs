extern crate linux_embedded_hal;
use linux_embedded_hal::spidev::{self, SpidevOptions};
use linux_embedded_hal::sysfs_gpio::Direction;
use linux_embedded_hal::Delay;
use linux_embedded_hal::{Pin, Spidev};

extern crate ssd1675;
use ssd1675::{Display, Dimensions, GraphicDisplay, Color, Rotation};

// Graphics
extern crate embedded_graphics;
use embedded_graphics::coord::Coord;
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;

// Font
extern crate profont;
use profont::{ProFont9Point, ProFont12Point, ProFont14Point, ProFont24Point};

use std::process::Command;
use std::{fs, io};
use std::time::Duration;
use std::thread::sleep;

// Activate SPI, GPIO in raspi-config needs to be run with sudo because of some sysfs_gpio
// permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

const ROWS: u16 = 212;
const COLS: u8 = 104;

fn main() -> Result<(), std::io::Error> {
    // Configure SPI
    let mut spi = Spidev::open("/dev/spidev0.0").expect("SPI device");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("SPI configuration");

    // https://pinout.xyz/pinout/inky_phat#
    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = Pin::new(8); // BCM8
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out).expect("CS Direction");
    cs.set_value(1).expect("CS Value set to 1");

    let busy = Pin::new(17); // BCM17
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In).expect("busy Direction");

    let dc = Pin::new(22); // BCM22
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out).expect("dc Direction");
    dc.set_value(1).expect("dc Value set to 1");

    let reset = Pin::new(27); // BCM27
    reset.export().expect("reset export");
    while !reset.is_exported() {}
    reset.set_direction(Direction::Out).expect("reset Direction");
    reset.set_value(1).expect("reset Value set to 1");
    println!("Pins configured");

    let mut delay = Delay {};

    let controller = ssd1675::Interface::new(spi, cs, busy, dc, reset);

    let dimensions = Dimensions { rows: ROWS, cols: COLS };
    let mut black_buffer = [0u8; ROWS as usize * COLS as usize / 8];
    let mut red_buffer = [0u8; ROWS as usize * COLS as usize / 8];
    let display = Display::new(controller, dimensions, Rotation::Rotate270);
    let mut display = GraphicDisplay::new(display, &mut black_buffer, &mut red_buffer);

    loop {
        display.reset(&mut delay).expect("error resetting display");
        println!("Reset and initialised");
        let one_minute = Duration::from_secs(60);

        display.clear(Color::White);
        println!("Clear");

        display.draw(
            ProFont24Point::render_str("Raspberry Pi")
                .with_stroke(Some(Color::Red))
                .with_fill(Some(Color::White))
                .translate(Coord::new(1, -4))
                .into_iter(),
        );

        if let Ok(cpu_temp) = read_cpu_temp() {
            display.draw(
                ProFont14Point::render_str("CPU Temp:")
                    .with_stroke(Some(Color::Black))
                    .with_fill(Some(Color::White))
                    .translate(Coord::new(1, 30))
                    .into_iter(),
            );
            display.draw(
                ProFont12Point::render_str(&format!("{:.1}°C", cpu_temp))
                    .with_stroke(Some(Color::Black))
                    .with_fill(Some(Color::White))
                    .translate(Coord::new(95, 34))
                    .into_iter(),
            );
        }

        if let Some(uptime) = read_uptime() {
            display.draw(
                ProFont9Point::render_str(uptime.trim())
                    .with_stroke(Some(Color::Black))
                    .with_fill(Some(Color::White))
                    .translate(Coord::new(1, 93))
                    .into_iter(),
            );
        }

        if let Some(uname) = read_uname() {
            display.draw(
                ProFont9Point::render_str(uname.trim())
                    .with_stroke(Some(Color::Black))
                    .with_fill(Some(Color::White))
                    .translate(Coord::new(1, 84))
                    .into_iter(),
            );
        }

        display.update(&mut delay).expect("error updating display");
        println!("Update...");

        println!("Finished - going to sleep");
        display.deep_sleep()?;

        sleep(one_minute);
    }
}

fn read_cpu_temp() -> Result<f64, io::Error> {
    fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?
        .trim()
        .parse::<i32>()
        .map(|temp| temp as f64 / 1000.)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

fn read_uptime() -> Option<String> {
    Command::new("uptime").arg("-p").output().ok().and_then(|output| {
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    })
}

fn read_uname() -> Option<String> {
    Command::new("uname").arg("-smr").output().ok().and_then(|output| {
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    })
}