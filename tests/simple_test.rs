extern crate failure;
extern crate libheif;

use failure::Error;

#[test]
fn create_context() -> Result<(), Error> {
    let mut ctx = libheif::Context::from_file("road.heic")?;
    let mut handle = ctx.get_primary_image()?;
    let mut image = handle.decode()?;
    let pixels = image.pixels()?;

    // 3a4042 is the hex of the top pixel, but.. what's the 3a? Next pixel?
    // That's not what we asked for...
    assert_eq!(0x3a40423a, pixels.get_four(0, 0));
    Ok(())
}
