#!/bin/bash

(cd c/libheif && bindgen \
  --whitelist-function 'heif_.*' \
  libheif/heif.h         \
  --                     \
  -I .                   \
  -I ../stubs            \
) > src/raw.rs
