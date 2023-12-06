use pdfium_render::{document::PdfDocument, pages::PdfPages, pdfium::Pdfium};
use sqlite_loadable::{
    api,
    table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    BestIndexError, Result,
};
use sqlite_loadable::{prelude::*, Error};

use std::{marker::PhantomData, mem, os::raw::c_int};

static CREATE_SQL: &str = "CREATE TABLE x(width int, height int, full_text text, page, pdf hidden)";
enum Columns {
    Width,
    Height,
    FullText,
    Page,
    Pdf,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Width),
        1 => Some(Columns::Height),
        2 => Some(Columns::FullText),
        3 => Some(Columns::Page),
        4 => Some(Columns::Pdf),
        _ => None,
    }
}

#[repr(C)]
pub struct PdfPagesTable {
    /// must be first
    base: sqlite3_vtab,
    pdfium: Pdfium,
}

impl<'vtab> VTab<'vtab> for PdfPagesTable {
    type Aux = Pdfium;
    type Cursor = PdfPagesCursor<'vtab>;

    fn connect(
        _db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PdfPagesTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = PdfPagesTable {
            base,
            pdfium: Pdfium::default(),
        };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_pdf = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Pdf) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_pdf = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => (),
            }
        }
        if !has_pdf {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(2);

        Ok(())
    }

    fn open(&mut self) -> Result<PdfPagesCursor<'_>> {
        Ok(PdfPagesCursor::new(&self.pdfium))
    }
}

type MMatch = (usize, usize, String);
#[repr(C)]
pub struct PdfPagesCursor<'vtab> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    pdfium: &'vtab Pdfium,
    rowid: u16,
    pdf_document: Option<PdfDocument<'vtab>>,
    pdf_pages: Option<&'vtab PdfPages<'vtab>>,
    phantom: PhantomData<&'vtab PdfPagesTable>,
}
impl PdfPagesCursor<'_> {
    fn new<'vtab>(pdfium: &'vtab Pdfium) -> PdfPagesCursor<'vtab> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PdfPagesCursor {
            base,
            pdfium,
            rowid: 0,
            pdf_document: None,
            pdf_pages: None,
            phantom: PhantomData,
        }
    }
}

impl VTabCursor for PdfPagesCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let src = api::value_blob(&values[0]);
        let pdf = self.pdfium.load_pdf_from_byte_slice(src, None).unwrap();
        //self.pdf_pages = Some(pages);
        self.rowid = 0;
        self.pdf_document = Some(pdf);

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        println!("next");
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.rowid >= self.pdf_document.as_ref().unwrap().pages().len()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::Width) => {
                let page = self
                    .pdf_document
                    .as_ref()
                    .unwrap()
                    .pages()
                    .get(self.rowid)
                    .unwrap();
                api::result_double(context, page.width().value.into());
            }
            Some(Columns::Height) => {
                let page = self
                    .pdf_document
                    .as_ref()
                    .unwrap()
                    .pages()
                    .get(self.rowid)
                    .unwrap();
                api::result_double(context, page.height().value.into());
            }
            Some(Columns::FullText) => {
                let page = self
                    .pdf_document
                    .as_ref()
                    .unwrap()
                    .pages()
                    .get(self.rowid)
                    .unwrap();
                api::result_text(context, page.text().unwrap().all())?;
            }
            Some(Columns::Page) => {
                let page = self
                    .pdf_document
                    .as_ref()
                    .unwrap()
                    .pages()
                    .get(self.rowid)
                    .unwrap();
                api::result_pointer(context, b"wut\0", page);
            }
            Some(Columns::Pdf) => {
                api::result_null(context);
            }
            None => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid.into())
    }
}
