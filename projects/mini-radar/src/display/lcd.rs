use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, OriginDimensions},
};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiBus};
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::options::{ColorInversion, ColorOrder};
use mipidsi::{Builder, interface::SpiInterface, models::GC9A01};

pub fn init<'a, SPI, CS, DC, RST, DELAY>(
    spi_bus: SPI,
    display_cs: CS,
    display_dc: DC,
    display_rst: RST,
    buffer: &'a mut [u8],
    delay: &mut DELAY,
) -> impl DrawTarget<Color = Rgb565, Error: core::fmt::Debug> + OriginDimensions + 'a
where
    SPI: SpiBus<u8> + 'a,
    CS: OutputPin + 'a,
    DC: OutputPin + 'a,
    RST: OutputPin + 'a,
    DELAY: DelayNs,
{
    // Moves SPI bus and CS into one logical SPI device.
    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, display_cs).unwrap();

    // Moves the SPI device and DC into the mipidsi interface.
    //The buffer is a temporary SPI transmission buffer owned by mipidsi’s SpiInterface.
    let interface = SpiInterface::new(spi_device, display_dc, buffer);

    // Moves the interface and reset pin into the GC9A01 driver.
    Builder::new(GC9A01, interface)
        .reset_pin(display_rst)
        .invert_colors(ColorInversion::Inverted)
        .color_order(ColorOrder::Bgr)
        .init(delay)
        .unwrap()
}
