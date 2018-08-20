extern crate failure;
extern crate libheif;

use failure::Error;
use libheif::simple::Channel;
use libheif::simple::Chroma;
use libheif::simple::ColourSpace;
use libheif::simple::Context;
use libheif::simple::DecoderSettings;

#[test]
fn first_pixel() -> Result<(), Error> {
    let mut ctx = Context::from_file("road.heic")?;
    let mut handle = ctx.get_primary_image()?;
    {
        let mut image = handle.decode(DecoderSettings::interleaved_rgb())?;
        let mut plane = image.plane(Channel::Interleaved)?;
        let pixels = plane.pixels()?;
        let row = pixels.get_row(0);

        assert_eq!(0x3a, row[0]);
        assert_eq!(0x42, row[1]);
        assert_eq!(0x40, row[2]);
    }

    {
        let mut image = handle.decode(DecoderSettings {
            chroma: Chroma::C420,
            colour_space: ColourSpace::YCbCr,
        })?;
        let mut plane = image.plane(Channel::Y)?;
        let pixels = plane.pixels()?;
        let row = pixels.get_row(0);

        // ?? no idea if this is right
        assert_eq!(0x40, row[0]);
    }

    Ok(())
}
