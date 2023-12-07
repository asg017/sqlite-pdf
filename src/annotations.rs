use pdfium_render::{
    document::PdfDocument,
    page_annotation::{PdfPageAnnotation, PdfPageAnnotationCommon, PdfPageAnnotationType},
    page_annotations::PdfPageAnnotationsIterator,
};
use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    api,
    table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    BestIndexError, Result,
};

use std::{marker::PhantomData, mem, os::raw::c_int};

use crate::PagePointer;

static CREATE_SQL: &str =
    "CREATE TABLE x(type, x, y, width, height, name, contents, creator, created_at, modified_at, page hidden)";
enum Columns {
    Type,
    X,
    Y,
    Width,
    Height,
    Name,
    Contents,
    Creator,
    CreatedAt,
    ModifiedAt,
    Page,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Type),
        1 => Some(Columns::X),
        2 => Some(Columns::Y),
        3 => Some(Columns::Width),
        4 => Some(Columns::Height),
        5 => Some(Columns::Name),
        6 => Some(Columns::Contents),
        7 => Some(Columns::Creator),
        8 => Some(Columns::CreatedAt),
        9 => Some(Columns::ModifiedAt),
        10 => Some(Columns::Page),
        _ => None,
    }
}

#[repr(C)]
pub struct PdfAnnotationsTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for PdfAnnotationsTable {
    type Aux = ();
    type Cursor = PdfAnnotationsCursor<'vtab>;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PdfAnnotationsTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = PdfAnnotationsTable { base };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
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

    fn open(&mut self) -> Result<PdfAnnotationsCursor<'_>> {
        Ok(PdfAnnotationsCursor::new())
    }
}

#[repr(C)]
pub struct PdfAnnotationsCursor<'vtab> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    document: Option<*const PdfDocument<'vtab>>,
    current: Option<PdfPageAnnotation<'vtab>>,
    iter: Option<PdfPageAnnotationsIterator<'vtab>>,
    phantom: PhantomData<&'vtab PdfAnnotationsTable>,
}
impl PdfAnnotationsCursor<'_> {
    fn new<'vtab>() -> PdfAnnotationsCursor<'vtab> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PdfAnnotationsCursor {
            base,
            rowid: 0,
            document: None,
            current: None,
            iter: None,
            phantom: PhantomData,
        }
    }
}

impl VTabCursor for PdfAnnotationsCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let page: *mut PagePointer = unsafe { api::value_pointer(&values[0], b"wut\0").unwrap() };
        unsafe {
            self.iter = Some((*page).1.annotations().iter());
        }

        self.rowid = 0;
        self.next()?;

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        self.current = self.iter.as_mut().unwrap().next();

        Ok(())
    }

    fn eof(&self) -> bool {
        self.current.is_none()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let annotation = self.current.as_ref().unwrap();
        //annotation.bounds()
        match column(i) {
            Some(Columns::Type) => {
                let typename = match annotation.annotation_type() {
                    PdfPageAnnotationType::Unknown => "unknown",
                    PdfPageAnnotationType::Text => "text",
                    PdfPageAnnotationType::Link => "link",
                    PdfPageAnnotationType::FreeText => "freetext",
                    PdfPageAnnotationType::Line => "line",
                    PdfPageAnnotationType::Square => "square",
                    PdfPageAnnotationType::Circle => "circle",
                    PdfPageAnnotationType::Polygon => "polygon",
                    PdfPageAnnotationType::Polyline => "polyline",
                    PdfPageAnnotationType::Highlight => "highlight",
                    PdfPageAnnotationType::Underline => "underline",
                    PdfPageAnnotationType::Squiggly => "squiggly",
                    PdfPageAnnotationType::Strikeout => "strikeout",
                    PdfPageAnnotationType::Stamp => "stamp",
                    PdfPageAnnotationType::Caret => "caret",
                    PdfPageAnnotationType::Ink => "ink",
                    PdfPageAnnotationType::Popup => "popup",
                    PdfPageAnnotationType::FileAttachment => "fileattachment",
                    PdfPageAnnotationType::Sound => "sound",
                    PdfPageAnnotationType::Movie => "movie",
                    PdfPageAnnotationType::Widget => "widget",
                    PdfPageAnnotationType::Screen => "screen",
                    PdfPageAnnotationType::PrinterMark => "printermark",
                    PdfPageAnnotationType::TrapNet => "trapnet",
                    PdfPageAnnotationType::Watermark => "watermark",
                    PdfPageAnnotationType::ThreeD => "threed",
                    PdfPageAnnotationType::RichMedia => "richmedia",
                    PdfPageAnnotationType::XfaWidget => "xfawidget",
                    PdfPageAnnotationType::Redacted => "redacted",
                };
                api::result_text(context, typename)?;
            }
            Some(Columns::X) => {
                api::result_double(context, annotation.bounds().unwrap().left.value.into())
            }
            Some(Columns::Y) => {
                api::result_double(context, annotation.bounds().unwrap().top.value.into())
            }
            Some(Columns::Width) => {
                api::result_double(context, annotation.bounds().unwrap().width().value.into())
            }
            Some(Columns::Height) => {
                api::result_double(context, annotation.bounds().unwrap().height().value.into())
            }
            Some(Columns::Name) => match annotation.name() {
                Some(name) => api::result_text(context, name)?,
                None => api::result_null(context),
            },
            Some(Columns::Contents) => match annotation.contents() {
                Some(contents) => api::result_text(context, contents)?,
                None => api::result_null(context),
            },
            Some(Columns::Creator) => match annotation.creator() {
                Some(creator) => api::result_text(context, creator)?,
                None => api::result_null(context),
            },
            Some(Columns::CreatedAt) => match annotation.creation_date() {
                Some(creation_date) => api::result_text(context, creation_date)?,
                None => api::result_null(context),
            },
            Some(Columns::ModifiedAt) => match annotation.modification_date() {
                Some(modification_date) => api::result_text(context, modification_date)?,
                None => api::result_null(context),
            },
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
