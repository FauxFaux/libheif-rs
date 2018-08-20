extern crate failure;
extern crate libheif;

use failure::Error;

#[test]
fn create_context() -> Result<(), Error> {
    let mut ctx = libheif::Context::from_file("road.heic")?;
    let mut handle = ctx.get_primary_image()?;
    handle.decode()?;
    Ok(())
}
