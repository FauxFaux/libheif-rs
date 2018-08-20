#[macro_use]
extern crate failure;
extern crate libheif;

use std::env;

use failure::Error;
use libheif::simple::Channel;
use libheif::simple::Context;
use libheif::simple::DecoderSettings;

fn main() -> Result<(), Error> {
    let mut ctx = Context::from_file(
        env::args_os()
            .nth(1)
            .ok_or_else(|| format_err!("usage: FILENAME"))?,
    )?;
    let mut handle = ctx.get_primary_image()?;
    let mut image = handle.decode(DecoderSettings::interleaved_rgb())?;

    let mut plane = image.plane(Channel::Interleaved)?;
    println!("w: {}, h: {}", plane.width(), plane.height());
    let pixels = plane.pixels()?;

    unimplemented!();
    Ok(())
}
