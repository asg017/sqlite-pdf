.timer on

.load dist/debug/pdf0

.mode box
.header on

select *
from pdf_pages(
  readfile('pdf_commenting_new.pdf')
);


select
  pdf_pages.rowid,
  pdf_annotations.*
from pdf_pages(
  readfile('pdf_commenting_new.pdf')
)
join pdf_annotations(pdf_pages.page);


--create table pages_demo as
select
  width,
  height,
  full_text,
  pdf_page_thumbnail(page)
from pdf_pages(
  readfile('signed_taranto_statement_of_facts_complaint_final_redacted_0.pdf')
);


--create table images_demo as
select
  pdf_pages.rowid as page_rowid,
  pdf_images.*
from pdf_pages(
  readfile('signed_taranto_statement_of_facts_complaint_final_redacted_0.pdf')
)
join pdf_images(pdf_pages.page);


