use pdfium_render::{
    document::PdfDocument,
    page::PdfPage,
    page_object::{PdfPageObject, PdfPageObjectCommon, PdfPageObjectType},
    page_objects_common::{PdfPageObjectsCommon, PdfPageObjectsIterator},
    pdfium::Pdfium,
};
use sqlite_loadable::{
    api,
    table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    BestIndexError, Result,
};
use sqlite_loadable::{prelude::*, Error};

use std::{marker::PhantomData, mem, os::raw::c_int};

static CREATE_SQL: &str = "CREATE TABLE x(x, y, width, height, image, page hidden)";
enum Columns {
    X,
    Y,
    Width,
    Height,
    Image,
    Page,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::X),
        1 => Some(Columns::Y),
        2 => Some(Columns::Width),
        3 => Some(Columns::Height),
        4 => Some(Columns::Image),
        5 => Some(Columns::Page),
        _ => None,
    }
}

#[repr(C)]
pub struct PdfImagesTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for PdfImagesTable {
    type Aux = Pdfium;
    type Cursor = PdfImagesCursor<'vtab>;

    fn connect(
        _db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PdfImagesTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = PdfImagesTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        println!("pdf_pages best_index");
        let mut has_page = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Page) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_page = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => (),
            }
        }
        if !has_page {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(2);

        Ok(())
    }

    fn open(&mut self) -> Result<PdfImagesCursor<'_>> {
        Ok(PdfImagesCursor::new())
    }
}

type MMatch = (usize, usize, String);
#[repr(C)]
pub struct PdfImagesCursor<'vtab> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    current: Option<PdfPageObject<'vtab>>,
    iter: Option<PdfPageObjectsIterator<'vtab>>,
    phantom: PhantomData<&'vtab PdfImagesTable>,
}
impl PdfImagesCursor<'_> {
    fn new<'vtab>() -> PdfImagesCursor<'vtab> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PdfImagesCursor {
            base,
            rowid: 0,
            current: None,
            iter: None,
            phantom: PhantomData,
        }
    }
}

impl VTabCursor for PdfImagesCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        println!("pdf_images");
        let page: *mut PdfPage = unsafe { api::value_pointer(&values[0], b"wut\0").unwrap() };
        unsafe {
            let o = (*page).objects().iter();
            self.iter = Some(o);
        }

        self.rowid = 0;
        self.next()?;

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        loop {
            self.current = self.iter.as_mut().unwrap().next();
            match self.current.as_ref() {
                None => break,
                Some(PdfPageObject::Image(_)) => break,
                _ => continue,
            }
        }
        Ok(())
    }

    fn eof(&self) -> bool {
        self.current.is_none()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let img = self.current.as_ref().unwrap().as_image_object().unwrap();
        match column(i) {
            Some(Columns::X) => {
                api::result_double(context, img.bounds().unwrap().left.value.into())
            }
            Some(Columns::Y) => api::result_double(context, img.bounds().unwrap().top.value.into()),
            Some(Columns::Width) => api::result_double(context, img.width().unwrap().value.into()),
            Some(Columns::Height) => {
                api::result_double(context, img.height().unwrap().value.into())
            }
            Some(Columns::Image) => {
                //img.get_processed_image();
            }

            Some(Columns::Page) => {
                api::result_null(context);
            }
            None => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}
