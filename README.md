# SSD1680 ePaper Display Driver

Rust driver for the [Solomon Systech SSD1680][SSD1680] e-Paper display (EPD)
controller, for use with [embedded-hal].

[![Build Status](https://travis-ci.org/Radiator-Labs/ssd1680.svg?branch=master)](https://travis-ci.org/Radiator-Labs/ssd1680)
[![codecov](https://codecov.io/gh/Radiator-Labs/ssd1680/branch/master/graph/badge.svg)](https://codecov.io/gh/Radiator-Labs/ssd1680)
<!-- [![crates.io](https://img.shields.io/crates/v/ssd1680.svg)](https://crates.io/crates/ssd1680) -->
<!-- [![Documentation](https://docs.rs/ssd1680/badge.svg)][crate-docs] -->

<img src="https://raw.githubusercontent.com/Radiator-Labs/ssd1680/master/IMG_2435.jpg" width="459" alt="Photo of GDEY029T94 ePaper display on STM32WL55 board" />

## Attribution

This driver is based on the [SSD1675 driver](https://github.com/wezm/ssd1675) by [wezm](https://github.com/wezm).
Work converting this driver to support the SSD1680 was performed as part of commercial
development by [Kelvin](https://kel.vin/) (formerly Radiator Labs), a green energy company
dedicated to decarbonizing the world's legacy buildings.

The open source license of the original project is retained for this driver.

## Description

This driver is intended to work on embedded platforms using the `embedded-hal`
trait library. It is `no_std` compatible, builds on stable Rust, and only uses
safe Rust. It supports the 4-wire SPI interface.

## Tested Devices

The library has been tested and confirmed working on these devices:

* Black/White [GDEY029T94] on Nucleo-STM32WL55 (pictured above)

## Obsoleted Examples

The examples have not been updated from the SSD1675 and are not expected to operate.

**Note:** To build the examples the `examples` feature needs to be enabled. E.g.

    cargo build --release --examples --features examples

### Raspberry Pi with Inky pHAT

The [Raspberry Pi Inky pHAT
example](https://github.com/wezm/ssd1675/blob/master/examples/raspberry_pi_inky_phat.rs),
shows how to display information on an [Inky pHAT] using this crate. The photo
at the top of the page shows this example in action. To avoid the need to
compile on the Raspberry Pi itself I recommend cross-compiling with the [cross]
tool. With `cross` installed build the example as follows:

    cross build --target=arm-unknown-linux-gnueabi --release --example raspberry_pi_inky_phat --features examples

After it is built copy
`target/arm-unknown-linux-gnueabi/release/examples/raspberry_pi_inky_phat` to
the Raspberry Pi.

## Credits

* [SSD1675 eInk display driver](https://github.com/wezm/ssd1675)
* [Waveshare EPD driver](https://github.com/caemor/epd-waveshare)
* [SSD1306 OLED display driver](https://github.com/jamwaffles/ssd1306)
* [SSD1322 OLED display driver](https://github.com/edarc/ssd1322)
* [Pimoroni Python library for the Inky pHAT and Inky wHAT e-paper displays](https://github.com/pimoroni/inky)

## License

`ssd1680` is dual licensed under:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) **or**
  <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

<!-- [crate-docs]: https://docs.rs/ssd1680 -->
[cross]: https://github.com/rust-embedded/cross
[embedded-hal]: https://crates.io/crates/embedded-hal
[Inky pHAT]: https://shop.pimoroni.com/products/inky-phat
[GDEY029T94]: https://www.good-display.com/product/389.html
[SSD1680]: http://www.solomon-systech.com/en/product/advanced-display/bistable-display-driver-ic/SSD1680/
