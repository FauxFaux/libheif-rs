extern crate cc;

use std::ffi;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let mut files = Vec::new();
    for base in &[Path::new("c/libheif/libheif"), Path::new("c/libde265/libde265")] {
        for entry in fs::read_dir(base)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().contains("fuzzer") {
                continue;
            }
            if let Some(extension) = Path::new(&entry.file_name()).extension() {
                if extension == "cc" {
                    let mut full_path = base.to_path_buf();
                    full_path.push(entry.file_name());
                    files.push(full_path);
                }
            }
        }
    }

    files.sort();

    cc::Build::new()
        .include("c/stubs")
        .include("c/libheif")
        .include("c/libde265")
        .define("HAVE_UNISTD_H", Some("1"))
        .define("HAVE_STDINT_H", Some("1"))
        .define("HAVE_MALLOC_H", Some("1"))
        .files(files)
        .cpp(true)
        .compile("heif-all");

    Ok(())
}
