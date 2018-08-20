use std::ffi::CStr;
use std::ffi::CString;
use std::os;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

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

pub struct Pixels<'c: 'h, 'h: 'i, 'i> {
    image: &'i mut Image<'c, 'h>,
    stride: usize,
    data: *const u8,
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
    pub fn decode<'s>(&'s mut self) -> Result<Image<'c, 's>, Error> {
        let mut ptr = ::std::ptr::null_mut();

        check_error("heif_decode_image", unsafe {
            raw::heif_decode_image(
                self.inner,
                &mut ptr,
                raw::heif_colorspace_heif_colorspace_RGB,
                raw::heif_chroma_heif_chroma_interleaved_RGB,
                ptr::null(),
            )
        })?;

        Ok(Image {
            handle: self,
            inner: ptr,
        })
    }
}

impl<'c, 'h> Image<'c, 'h> {
    pub fn pixels<'s>(&'s mut self) -> Result<Pixels<'c, 'h, 's>, Error> {
        let mut stride: os::raw::c_int = 0;

        let ptr = unsafe {
            raw::heif_image_get_plane_readonly(
                self.inner,
                raw::heif_channel_heif_channel_interleaved,
                &mut stride,
            )
        };

        if ptr.is_null() {
            bail!("heif_image_get_plane_readonly failed");
        }

        Ok(Pixels {
            image: self,
            stride: usize(stride)?,
            data: ptr,
        })
    }
}

impl<'c, 'h, 'i> Pixels<'c, 'h, 'i> {
    pub fn get_four(&self, x: usize, y: usize) -> u32 {
        assert_eq!(0, y);
        // TODO: self.step
        // TODO: width
        // TODO: width validation
        unsafe { *(self.data.offset(isize(x).expect("too big for isize")) as *const u32) }
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
