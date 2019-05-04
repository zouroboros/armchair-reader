use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_double;
use std::os::raw::c_void;
use std::os::raw::{c_char, c_int};
use std::path;

extern crate cairo;

extern crate cairo_sys;
extern crate glib;
extern crate glib_sys;

mod ffi;
mod util;

#[derive(Debug)]
pub struct PopplerDocument(*mut ffi::PopplerDocument);

#[derive(Debug)]
pub struct PopplerPage(*mut ffi::PopplerPage);

impl PopplerDocument {
    pub fn new_from_file<P: AsRef<path::Path>>(
        p: P,
        password: &str,
    ) -> Result<PopplerDocument, glib::error::Error> {
        let pw = CString::new(password).map_err(|_| {
            glib::error::Error::new(
                glib::FileError::Inval,
                "Password invalid (possibly contains NUL characters)",
            )
        })?;

        let path_cstring = util::path_to_glib_url(p)?;
        let doc = util::call_with_gerror(|err_ptr| unsafe {
            ffi::poppler_document_new_from_file(path_cstring.as_ptr(), pw.as_ptr(), err_ptr)
        })?;

        Ok(PopplerDocument(doc))
    }
    pub fn new_from_data(
        data: &[u8],
        password: &str,
    ) -> Result<PopplerDocument, glib::error::Error> {
        if data.len() == 0 {
            return Err(glib::error::Error::new(
                glib::FileError::Inval,
                "data is empty",
            ));
        }
        let pw = CString::new(password).map_err(|_| {
            glib::error::Error::new(
                glib::FileError::Inval,
                "Password invalid (possibly contains NUL characters)",
            )
        })?;

        let doc = util::call_with_gerror(|err_ptr| unsafe {
            ffi::poppler_document_new_from_data(
                data.as_ptr() as *const c_char,
                data.len() as c_int,
                pw.as_ptr(),
                err_ptr,
            )
        })?;

        Ok(PopplerDocument(doc))
    }
    pub fn get_title(&self) -> Option<String> {
        unsafe {
            let ptr: *mut c_char = ffi::poppler_document_get_title(self.0);
            if ptr.is_null() {
                None
            } else {
                CString::from_raw(ptr).into_string().ok()
            }
        }
    }
    pub fn get_metadata(&self) -> Option<String> {
        unsafe {
            let ptr: *mut c_char = ffi::poppler_document_get_metadata(self.0);
            if ptr.is_null() {
                None
            } else {
                CString::from_raw(ptr).into_string().ok()
            }
        }
    }
    pub fn get_pdf_version_string(&self) -> Option<String> {
        unsafe {
            let ptr: *mut c_char = ffi::poppler_document_get_pdf_version_string(self.0);
            if ptr.is_null() {
                None
            } else {
                CString::from_raw(ptr).into_string().ok()
            }
        }
    }
    pub fn get_permissions(&self) -> u8 {
        unsafe { ffi::poppler_document_get_permissions(self.0) as u8 }
    }

    pub fn get_n_pages(&self) -> usize {
        // FIXME: what's the correct type here? can we assume a document
        //        has a positive number of pages?
        (unsafe { ffi::poppler_document_get_n_pages(self.0) }) as usize
    }

    pub fn get_page(&self, index: usize) -> Option<PopplerPage> {
        match unsafe { ffi::poppler_document_get_page(self.0, index as c_int) } {
            ptr if ptr.is_null() => None,
            ptr => Some(PopplerPage(ptr)),
        }
    }
}

impl PopplerPage {
    pub fn get_size(&self) -> (f64, f64) {
        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;

        unsafe {
            ffi::poppler_page_get_size(
                self.0,
                &mut width as *mut f64 as *mut c_double,
                &mut height as *mut f64 as *mut c_double,
            )
        }

        (width, height)
    }

    pub fn render(&self, ctx: &cairo::Context) {
        unsafe { ffi::poppler_page_render(self.0, ctx.to_raw_none()) }
    }

    pub fn render_for_printing(&self, ctx: &mut cairo::Context) {
        unsafe { ffi::poppler_page_render_for_printing(self.0, ctx.to_raw_none()) }
    }

    pub fn get_text(&self) -> Option<&str> {
        match unsafe { ffi::poppler_page_get_text(self.0) } {
            ptr if ptr.is_null() => None,
            ptr => unsafe { Some(CStr::from_ptr(ptr).to_str().unwrap()) },
        }
    }
}

// FIXME: needs to be in upstream version of cairo-rs
pub trait CairoSetSize {
    fn set_size(&mut self, width_in_points: f64, height_in_points: f64);
}

impl CairoSetSize for cairo::Surface {
    // FIXME: does this need mut?
    fn set_size(&mut self, width_in_points: f64, height_in_points: f64) {
        unsafe {
            ffi::cairo_pdf_surface_set_size(
                self.to_raw_none(),
                width_in_points as c_double,
                height_in_points as c_double,
            )
        }
    }
}

#[derive(Debug)]
pub struct PoppperPageRef {
    ptr: *mut c_void,
}

#[cfg(test)]
mod tests {
    use cairo::enums::Format::ARgb32;
    use cairo::prelude::SurfaceExt;
    use cairo::Context;
    use cairo::ImageSurface;
    use cairo::PDFSurface;
    use std::{fs::File, io::Read};
    use CairoSetSize;
    use PopplerDocument;
    use PopplerPage;

    #[test]
    fn test1() {
        let filename = "test.pdf";
        let doc = PopplerDocument::new_from_file(filename, "").unwrap();
        let num_pages = doc.get_n_pages();

        println!("Document has {} page(s)", num_pages);

        let mut surface = PDFSurface::create("output.pdf", 420.0, 595.0);
        let mut ctx = Context::new(&mut surface);

        // FIXME: move iterator to poppler
        for page_num in 0..num_pages {
            let page = doc.get_page(page_num).unwrap();
            let (w, h) = page.get_size();
            println!("page {} has size {}, {}", page_num, w, h);
            surface.set_size(w, h);

            ctx.save();
            page.render(&mut ctx);

            println!("Text: {:?}", page.get_text().unwrap_or(""));

            ctx.restore();
            ctx.show_page();
        }
        // g_object_unref (page);
        //surface.write_to_png("file.png");
        surface.finish();
    }

    #[test]
    fn test2_from_file() {
        let path = "test.pdf";
        let doc: PopplerDocument = PopplerDocument::new_from_file(path, "upw").unwrap();
        let num_pages = doc.get_n_pages();
        let title = doc.get_title().unwrap();
        let metadata = doc.get_metadata();
        let version_string = doc.get_pdf_version_string();
        let permissions = doc.get_permissions();
        let page: PopplerPage = doc.get_page(0).unwrap();
        let (w, h) = page.get_size();

        println!(
            "Document {} has {} page(s) and is {}x{}",
            title, num_pages, w, h
        );
        println!(
            "Version: {:?}, Permissions: {:x?}",
            version_string, permissions
        );

        assert!(metadata.is_some());
        assert_eq!(version_string, Some("PDF-1.3".to_string()));
        assert_eq!(permissions, 0xff);

        assert_eq!(title, "This is a test PDF file");

        let mut surface = ImageSurface::create(ARgb32,  w as i32, h as i32).unwrap();
        let mut ctx = Context::new(&mut surface);

        ctx.save();
        page.render(&mut ctx);
        ctx.restore();
        ctx.show_page();

        let mut f: File = File::create("out.png").unwrap();
        surface.write_to_png(&mut f).expect("Unable to write PNG");
    }
    #[test]
    fn test2_from_data() {
        let path = "test.pdf";
        let mut file = File::open(path).unwrap();
        let mut data: Vec<u8> = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let doc: PopplerDocument = PopplerDocument::new_from_data(&data[..], "upw").unwrap();
        let num_pages = doc.get_n_pages();
        let title = doc.get_title().unwrap();
        let metadata = doc.get_metadata();
        let version_string = doc.get_pdf_version_string();
        let permissions = doc.get_permissions();
        let page: PopplerPage = doc.get_page(0).unwrap();
        let (w, h) = page.get_size();

        println!(
            "Document {} has {} page(s) and is {}x{}",
            title, num_pages, w, h
        );
        println!(
            "Version: {:?}, Permissions: {:x?}",
            version_string, permissions
        );

        assert!(metadata.is_some());
        assert_eq!(version_string, Some("PDF-1.3".to_string()));
        assert_eq!(permissions, 0xff);
    }

    #[test]
    fn test3() {
        let data = vec![];

        assert!(PopplerDocument::new_from_data(&data[..], "upw").is_err());
    }
}
