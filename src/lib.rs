mod images;
mod pages;
use std::io::{Read, Seek};

use image::ImageOutputFormat;
use pdfium_render::prelude::*;

use sqlite_loadable::{api, define_scalar_function, Result};
use sqlite_loadable::{define_table_function, prelude::*};

pub fn pdf_page_thumbnail(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    let page: *mut PdfPage = unsafe { api::value_pointer(&values[0], b"wut\0").unwrap() };
    let cfg = PdfRenderConfig::new().thumbnail(256);
    unsafe {
        let bitmap = (*page).render_with_config(&cfg).unwrap();
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

#[sqlite_entrypoint]
pub fn sqlite3_hello_init(db: *mut sqlite3) -> Result<()> {
    // For general comments about pdfium-render and binding to Pdfium, see export.rs.

    /*
    let pdf = pdfium.load_pdf_from_file("test.pdf", None).unwrap();
    pdf.pages().iter().enumerate().for_each(|(index, page)| {
        println!("=============== Page {} ===============", index);
        let cfg = PdfRenderConfig::new().thumbnail(800);
        let bitmap = page.render_with_config(&cfg).unwrap();
        bitmap
            .as_image()
            .save_with_format("file.png", image::ImageFormat::Png)
            .unwrap();
        for object in page.objects().iter() {
            match &object {
                PdfPageObject::Text(t) => {
                    println!("text: {} at {}", t.text(), t.bounds().unwrap())
                }
                PdfPageObject::Path(path) => {
                    println!("path: {:?}", path.segments().len())
                }
                PdfPageObject::Image(image) => {
                    println!("image: {:?}x{:?}", image.width(), image.height())
                }
                PdfPageObject::Shading(_) => todo!("Shading"),
                PdfPageObject::XObjectForm(_) => todo!("XObjectForm"),
                PdfPageObject::Unsupported(_) => todo!("Unsupported"),
            }
        }
    });
    */
    define_scalar_function(
        db,
        "pdf_page_thumbnail",
        1,
        pdf_page_thumbnail,
        FunctionFlags::DETERMINISTIC,
    )?;
    define_table_function::<pages::PdfPagesTable>(db, "pdf_pages", None)?;
    define_table_function::<images::PdfImagesTable>(db, "pdf_images", None)?;
    Ok(())
}
