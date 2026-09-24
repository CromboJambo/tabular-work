use std::fs;
use std::io::Cursor;
use zip::{ZipArchive, ZipWriter};

fn main() {
    // Create a minimal xlsx file programmatically using the zip crate
    let mut cursor = Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(&mut cursor);

    // [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#;
    archive.add_file("[Content_Types].xml", Default::default()).unwrap();
    archive.write(content_types.as_bytes()).unwrap();

    // _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;
    archive.add_file("_rels/.rels", Default::default()).unwrap();
    archive.write(rels.as_bytes()).unwrap();

    // xl/workbook.xml
    let workbook = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheets>
    <sheet name="Test Sheet" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#;
    archive.add_file("xl/workbook.xml", Default::default()).unwrap();
    archive.write(workbook.as_bytes()).unwrap();

    // xl/worksheets/sheet1.xml (empty sheet)
    let sheet = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData/>
</worksheet>"#;
    archive.add_file("xl/worksheets/sheet1.xml", Default::default()).unwrap();
    archive.write(sheet.as_bytes()).unwrap();

    // xl/_rels/workbook.xml.rels
    let wb_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#;
    archive.add_file("xl/_rels/workbook.xml.rels", Default::default()).unwrap();
    archive.write(wb_rels.as_bytes()).unwrap();

    archive.finish().unwrap();
    let bytes = cursor.into_inner();

    fs::write("examples/test_programmatic.xlsx", bytes).unwrap();
    println!("Created examples/test_programmatic.xlsx");
}