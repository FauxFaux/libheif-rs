use std::ffi::CStr;
use std::ffi::CString;
use std::os;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::slice;

use cast::isize;
use cast::usize;
use failure::Error;

use raw;

pub struct Context {
    inner: *mut raw::heif_context,
}

pub struct ImageHandle<'c> {
    ctx: &'c mut Context,
    inner: *mut raw::heif_image_handle,
}

pub struct Image<'c: 'h, 'h> {
    handle: &'h mut ImageHandle<'c>,
    inner: *mut raw::heif_image,
}

pub struct Plane<'c: 'h, 'h: 'i, 'i> {
    image: &'i mut Image<'c, 'h>,
    channel: Channel,
    width: usize,
    height: usize,
}

pub struct Pixels<'c: 'h, 'h: 'i, 'i: 'p, 'p> {
    plane: &'p mut Plane<'c, 'h, 'i>,
    stride: usize,
    data: *const u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DecoderSettings {
    pub chroma: Chroma,
    pub colour_space: ColourSpace,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Channel {
    Y,
    Cb,
    Cr,
    R,
    G,
    B,
    Alpha,
    Interleaved,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chroma {
    Undefined,
    Monochrome,
    C420,
    C422,
    C444,
    InterleavedRgb,
    InterleavedRgba,
    Other(raw::heif_chroma),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColourSpace {
    Undefined,
    YCbCr,
    Rgb,
    Monochrome,
    Other(raw::heif_colorspace),
}

impl Context {
    fn alloc() -> Result<*mut raw::heif_context, Error> {
        let ptr = unsafe { raw::heif_context_alloc() };
        if ptr.is_null() {
            bail!("allocation failed");
        }

        Ok(ptr)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let ptr = Context::alloc()?;
        let filename = CString::new(path.as_os_str().to_str().ok_or_else(|| {
            format_err!("path contained unrepresentable characters: {:?}", path)
        })?)?;

        check_error("heif_context_read_from_file", unsafe {
            raw::heif_context_read_from_file(ptr, filename.as_ptr(), ::std::ptr::null())
        })?;

        Ok(Context { inner: ptr })
    }

    pub fn get_primary_image(&mut self) -> Result<ImageHandle, Error> {
        let mut ptr = ::std::ptr::null_mut();
        check_error("heif_context_get_primary_image_handle", unsafe {
            raw::heif_context_get_primary_image_handle(self.inner, &mut ptr)
        })?;

        Ok(ImageHandle {
            ctx: self,
            inner: ptr,
        })
    }
}

impl<'c> ImageHandle<'c> {
    pub fn decode<'s>(&'s mut self, decode: DecoderSettings) -> Result<Image<'c, 's>, Error> {
        let mut ptr = ::std::ptr::null_mut();

        check_error("heif_decode_image", unsafe {
            raw::heif_decode_image(
                self.inner,
                &mut ptr,
                decode.colour_space.to_native(),
                decode.chroma.to_native(),
                ptr::null(),
            )
        })?;

        Ok(Image {
            handle: self,
            inner: ptr,
        })
    }

    pub fn width(&self) -> Result<usize, Error> {
        Ok(usize(unsafe {
            raw::heif_image_handle_get_width(self.inner)
        })?)
    }

    pub fn height(&self) -> Result<usize, Error> {
        Ok(usize(unsafe {
            raw::heif_image_handle_get_height(self.inner)
        })?)
    }
}

impl<'c, 'h> Image<'c, 'h> {
    pub fn plane<'s>(&'s mut self, channel: Channel) -> Result<Plane<'c, 'h, 's>, Error> {
        let native_channel = channel.to_native();

        if 0 == unsafe { raw::heif_image_has_channel(self.inner, native_channel) } {
            bail!("no such channel {:?}", channel);
        }
        let width = usize(unsafe { raw::heif_image_get_width(self.inner, native_channel) })?;
        let height = usize(unsafe { raw::heif_image_get_height(self.inner, native_channel) })?;
        Ok(Plane {
            image: self,
            channel,
            width,
            height,
        })
    }
}

impl<'c, 'h, 'i> Plane<'c, 'h, 'i> {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels<'s>(&'s mut self) -> Result<Pixels<'c, 'h, 'i, 's>, Error> {
        let mut stride: os::raw::c_int = 0;

        let ptr = unsafe {
            raw::heif_image_get_plane_readonly(
                self.image.inner,
                self.channel.to_native(),
                &mut stride,
            )
        };

        if ptr.is_null() {
            bail!("heif_image_get_plane_readonly failed");
        }
        let stride = usize(stride)?;
        assert_le!(self.width(), stride);

        Ok(Pixels {
            plane: self,
            stride,
            data: ptr,
        })
    }
}

impl<'c, 'h, 'i, 'p> Pixels<'c, 'h, 'i, 'p> {
    pub fn get_row(&self, y: usize) -> &[u8] {
        assert_lt!(y, self.plane.height());
        unsafe {
            slice::from_raw_parts(
                self.data
                    .offset(isize(y * self.stride).expect("too big for isize")),
                self.stride,
            )
        }
    }

    pub fn get_four(&self, x: usize, y: usize) -> u32 {
        assert_lt!(x, self.plane.width());
        assert_lt!(y, self.plane.height());
        let channel_bytes: usize = unimplemented!("channel_bytes");
        let off: usize = self.stride * y + x * channel_bytes;
        unsafe { *(self.data.offset(isize(off).expect("too big for isize")) as *const u32) }
    }
}

impl DecoderSettings {
    pub fn interleaved_rgb() -> DecoderSettings {
        DecoderSettings {
            chroma: Chroma::InterleavedRgb,
            colour_space: ColourSpace::Rgb,
        }
    }
}

#[inline]
fn check_error(location: &'static str, err: raw::heif_error) -> Result<(), Error> {
    if 0 == err.code {
        return Ok(());
    }

    bail!(
        "{}: {}/{}: {}",
        location,
        err.code,
        err.subcode,
        from_string_lossy(err.message)
    )
}

fn from_string_lossy(string: *const i8) -> String {
    unsafe { CStr::from_ptr(string) }
        .to_string_lossy()
        .to_string()
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { raw::heif_context_free(self.inner) }
    }
}

impl<'c> Drop for ImageHandle<'c> {
    fn drop(&mut self) {
        unsafe { raw::heif_image_handle_release(self.inner) }
    }
}

impl<'c, 'h> Drop for Image<'c, 'h> {
    fn drop(&mut self) {
        unsafe { raw::heif_image_release(self.inner) }
    }
}

impl Channel {
    fn to_native(&self) -> raw::heif_channel {
        match self {
            Channel::Y => raw::heif_channel_heif_channel_Y,
            Channel::Cb => raw::heif_channel_heif_channel_Cb,
            Channel::Cr => raw::heif_channel_heif_channel_Cr,
            Channel::R => raw::heif_channel_heif_channel_R,
            Channel::G => raw::heif_channel_heif_channel_G,
            Channel::B => raw::heif_channel_heif_channel_B,
            Channel::Alpha => raw::heif_channel_heif_channel_Alpha,
            Channel::Interleaved => raw::heif_channel_heif_channel_interleaved,
        }
    }
}

impl Chroma {
    fn to_native(&self) -> raw::heif_chroma {
        match self {
            Chroma::Undefined => raw::heif_chroma_heif_chroma_undefined,
            Chroma::Monochrome => raw::heif_chroma_heif_chroma_monochrome,
            Chroma::C420 => raw::heif_chroma_heif_chroma_420,
            Chroma::C422 => raw::heif_chroma_heif_chroma_422,
            Chroma::C444 => raw::heif_chroma_heif_chroma_444,
            Chroma::InterleavedRgb => raw::heif_chroma_heif_chroma_interleaved_RGB,
            Chroma::InterleavedRgba => raw::heif_chroma_heif_chroma_interleaved_RGBA,
            Chroma::Other(raw) => *raw,
        }
    }
}

impl ColourSpace {
    fn to_native(&self) -> raw::heif_colorspace {
        match self {
            ColourSpace::Undefined => raw::heif_colorspace_heif_colorspace_undefined,
            ColourSpace::YCbCr => raw::heif_colorspace_heif_colorspace_YCbCr,
            ColourSpace::Rgb => raw::heif_colorspace_heif_colorspace_RGB,
            ColourSpace::Monochrome => raw::heif_colorspace_heif_colorspace_monochrome,
            ColourSpace::Other(raw) => *raw,
        }
    }
}
