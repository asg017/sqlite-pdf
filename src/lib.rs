mod annotations;
mod images;
mod pages;
use std::io::{Read, Seek};

use image::ImageOutputFormat;
use pdfium_render::prelude::*;

use sqlite_loadable::{api, define_scalar_function, Result};
use sqlite_loadable::{define_table_function, prelude::*};

type PagePointer<'a> = (*const PdfDocument<'a>, PdfPage<'a>);
pub fn pdf_page_thumbnail(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    let xx: *mut PagePointer = unsafe { api::value_pointer(&values[0], b"wut\0").unwrap() };

    let cfg = PdfRenderConfig::new().thumbnail(256);
    unsafe {
        let bitmap = (*xx).1.render_with_config(&cfg).unwrap();
        let mut c = std::io::Cursor::new(Vec::new());
        bitmap
            .as_image()
            .write_to(&mut c, ImageOutputFormat::Png)
            .unwrap();
        let mut buffer = Vec::new();
        c.seek(std::io::SeekFrom::Start(0)).unwrap();
        c.read_to_end(&mut buffer).unwrap();

        api::result_blob(context, buffer.as_slice());
    }
    Ok(())
}
pub fn pdf_page_xxx(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let xx: *mut PagePointer = unsafe { api::value_pointer(&values[0], b"wut\0").unwrap() };

    unsafe {
        let p = Pdfium::default();
        let x = p.create_new_pdf().unwrap();
        //x.pages().copy_pages_from_document(source, pages, destination_page_index)
    }
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_pdf_init(db: *mut sqlite3) -> Result<()> {
    define_scalar_function(
        db,
        "pdf_page_thumbnail",
        1,
        pdf_page_thumbnail,
        FunctionFlags::DETERMINISTIC,
    )?;
    define_table_function::<pages::PdfPagesTable>(db, "pdf_pages", None)?;
    define_table_function::<images::PdfImagesTable>(db, "pdf_images", None)?;
    define_table_function::<annotations::PdfAnnotationsTable>(db, "pdf_annotations", None)?;
    Ok(())
}
