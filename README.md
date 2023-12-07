# sqlite-pdf

Work in progress! Will eventually be a SQLite extension for reading and writing PDFs.

```sql
select
  width,
  height,
  full_text,
  pdf_page_thumbnail(page, 256) as thumbnail
from pdf_pages(
  readfile('My Cool PDF.pdf')
);
```
