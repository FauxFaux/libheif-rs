use failure::Error;
use raw;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

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
