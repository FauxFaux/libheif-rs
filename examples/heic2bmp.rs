#[macro_use]
extern crate failure;
extern crate libheif;

use std::env;

use failure::Error;

fn main() -> Result<(), Error> {
    let mut ctx = libheif::Context::from_file(env::args_os().nth(1).ok_or_else(|| format_err!("usage: FILENAME"))?)?;
    let mut handle = ctx.get_primary_image()?;
    let mut image = handle.decode()?;
    let pixels = image.pixels()?;

    // 3a4042 is the hex of the top pixel, but.. what's the 3a? Next pixel?
    // That's not what we asked for...
    assert_eq!(0x3a40423a, pixels.get_four(0, 0));
    Ok(())
}
