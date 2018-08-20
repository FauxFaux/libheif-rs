#[macro_use]
extern crate failure;
extern crate libheif;

use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path;

use failure::Error;
use libheif::simple::Channel;
use libheif::simple::Context;
use libheif::simple::DecoderSettings;

fn main() -> Result<(), Error> {
    let usage = || format_err!("usage: FROM TO");
    let mut ctx = Context::from_file(env::args_os().nth(1).ok_or_else(usage)?)?;

    let mut handle = ctx.get_primary_image()?;
    let mut image = handle.decode(DecoderSettings::interleaved_rgb())?;

    let mut plane = image.plane(Channel::Interleaved)?;
    let pixels = plane.pixels()?;

    let mut dest = io::BufWriter::new(fs::File::create(env::args_os().nth(2).ok_or_else(usage)?)?);

    write!(dest, "P6\n{} {}\n255\n", plane.width(), plane.height())?;
    for row in 0..plane.height() {
        dest.write_all(&pixels.get_row(row)[..plane.width() * 3])?;
    }

    Ok(())
}
