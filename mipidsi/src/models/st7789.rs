use display_interface::{DataFormat, WriteOnlyDataCommand};
use embedded_graphics_core::{pixelcolor::Rgb565, prelude::IntoStorage};
use embedded_hal::{delay::DelayNs, digital::OutputPin};

use crate::{
    dcs::{
        BitsPerPixel, Dcs, EnterNormalMode, ExitSleepMode, PixelFormat, SetAddressMode,
        SetDisplayOn, SetInvertMode, SetPixelFormat, SoftReset, WriteMemoryStart,
    },
    error::{Error, InitError},
    models::Model,
    options::ModelOptions,
};

/// ST7789 display in Rgb565 color mode.
///
/// Interfaces implemented by the [display-interface](https://crates.io/crates/display-interface) are supported.
pub struct ST7789;

impl ST7789 {
    /// Write pixels from a u16 buffer, assuming they are in the correct format
    pub fn write_pixels_from_buf<DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        pixels_buf: &mut [u8],
    ) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
    {
        dcs.write_command(WriteMemoryStart)?;

        // let buf = DataFormat::U16LE(pixels_buf);
        let buf = DataFormat::U8(pixels_buf);
        dcs.di.send_data(buf)?;
        Ok(())
    }
}

impl Model for ST7789 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (240, 320);

    fn init<RST, DELAY, DI>(
        &mut self,
        dcs: &mut Dcs<DI>,
        delay: &mut DELAY,
        options: &ModelOptions,
        rst: &mut Option<RST>,
    ) -> Result<SetAddressMode, InitError<RST::Error>>
    where
        RST: OutputPin,
        DELAY: DelayNs,
        DI: WriteOnlyDataCommand,
    {
        let madctl = SetAddressMode::from(options);

        match rst {
            Some(ref mut rst) => self.hard_reset(rst, delay)?,
            None => dcs.write_command(SoftReset)?,
        }
        delay.delay_us(150_000);

        dcs.write_command(ExitSleepMode)?;
        delay.delay_us(10_000);

        // set hw scroll area based on framebuffer size
        dcs.write_command(madctl)?;

        dcs.write_command(SetInvertMode::new(options.invert_colors))?;

        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        dcs.write_command(SetPixelFormat::new(pf))?;
        delay.delay_us(10_000);
        dcs.write_command(EnterNormalMode)?;
        delay.delay_us(10_000);
        dcs.write_command(SetDisplayOn)?;

        // DISPON requires some time otherwise we risk SPI data issues
        delay.delay_us(120_000);

        Ok(madctl)
    }

    fn write_pixels<DI, I>(&mut self, dcs: &mut Dcs<DI>, colors: I) -> Result<(), Error>
    where
        DI: WriteOnlyDataCommand,
        I: IntoIterator<Item = Self::ColorFormat>,
    {
        dcs.write_command(WriteMemoryStart)?;

        let mut iter = colors.into_iter().map(Rgb565::into_storage);

        let buf = DataFormat::U16BEIter(&mut iter);
        dcs.di.send_data(buf)?;
        Ok(())
    }
}
