# poppler-rs
![poppler](https://img.shields.io/crates/v/poppler.svg)

[libpoppler](https://poppler.freedesktop.org/) is a library for rendering PDF files, this are Rust bindings to it. It uses [cairo](https://crates.io/crates/cairo-rs) for rendering, as a result PDF content can be drawn onto a number of surfaces, including SVG, PDF or PNG.

**Warning**: libpoppler is based on the GPL-licensed [xpdf-3.0](http://www.foolabs.com/xpdf/) and is unlikely to ever be released under a different license. As a result, every program or library linking against this crate *must* be GPL licensed as well.

The crate has only been tested on Linux; ensure that `libpoppler-glib` is installed to use it.
