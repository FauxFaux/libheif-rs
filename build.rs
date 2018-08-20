extern crate cc;

use std::io;
use std::path::Path;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let mut files = Vec::new();
    files.extend(extract_files("c/libheif/libheif", LIBHEIF_SOURCES));
    files.extend(extract_files("c/libde265/libde265", LIBDE265_SOURCES));

    files.sort();

    cc::Build::new()
        .include("c/stubs")
        .include("c/libheif")
        .include("c/libde265")
        .define("HAVE_LIBDE265", Some("1"))
        .define("HAVE_UNISTD_H", Some("1"))
        .define("HAVE_STDINT_H", Some("1"))
        .define("HAVE_MALLOC_H", Some("1"))
        .files(files)
        .warnings(false)
        .cpp(true)
        .compile("heif-all");

    Ok(())
}

// libheif/CMakeLists.txt
const LIBHEIF_SOURCES: &str = "
bitstream.h
bitstream.cc
box.cc
box.h
error.cc
error.h
heif_api_structs.h
heif.cc
heif_context.cc
heif_context.h
heif_file.cc
heif_file.h
heif.h
heif_image.cc
heif_image.h
heif_hevc.h
heif_hevc.cc
heif_plugin_registry.h
heif_plugin_registry.cc
heif_limits.h
heif_plugin.h
heif_plugin.cc
heif_version.h
logging.h
    heif_decoder_libde265.cc
    heif_decoder_libde265.h
";

// libde265/CMakeLists.txt
const LIBDE265_SOURCES: &str = "
  bitstream.cc
  cabac.cc
  de265.cc
  deblock.cc
  decctx.cc
  nal-parser.cc
  nal-parser.h
  dpb.cc
  dpb.h
  image.cc
  intrapred.cc
  md5.cc
  nal.cc
  pps.cc
  transform.cc
  refpic.cc
  sao.cc
  scan.cc
  sei.cc
  slice.cc
  sps.cc
  util.cc
  vps.cc
  bitstream.h
  cabac.h
  deblock.h
  decctx.h
  image.h
  intrapred.h
  md5.h
  nal.h
  pps.h
  transform.h
  refpic.h
  sao.h
  scan.h
  sei.h
  slice.h
  sps.h
  util.h
  vps.h
  vui.h vui.cc
  motion.cc motion.h
  threads.cc threads.h
  visualize.cc visualize.h
  acceleration.h
  fallback.cc fallback.h fallback-motion.cc fallback-motion.h
  fallback-dct.h fallback-dct.cc
  quality.cc quality.h
  configparam.cc configparam.h
  image-io.h image-io.cc
  alloc_pool.h alloc_pool.cc
  en265.h en265.cc
  contextmodel.cc
";

fn extract_files<P: AsRef<Path>>(base: P, paths: &str) -> Vec<PathBuf> {
    let base = base.as_ref();
    let mut files = Vec::new();
    for src in paths.split(|c: char| c.is_whitespace()) {
        let src = src.trim();
        if !src.ends_with(".cc") {
            continue;
        }
        let mut full_path = base.to_path_buf();
        full_path.push(src);
        files.push(full_path);
    }
    files
}
