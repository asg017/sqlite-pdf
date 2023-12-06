.timer on

.load target/debug/libsqlite_pdfium.dylib sqlite3_hello_init

.mode box
.header on

select
  pdf_pages.rowid,
  pdf_images.*
from pdf_pages(
  readfile('signed_taranto_statement_of_facts_complaint_final_redacted_0.pdf')
)
join pdf_images(pdf_pages.page)
limit 10;

.exit

create table t as
select
  width,
  height,
  full_text,
  pdf_page_thumbnail(page)
from pdf_pages(
  readfile('signed_taranto_statement_of_facts_complaint_final_redacted_0.pdf')
);
