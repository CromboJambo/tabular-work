use calamine::Reader;
fn main() {
    let f = std::fs::File::open("DATA/ToExcel_ServiceOrderTransactions_LABOR_ACCR_REC.xlsx").unwrap();
    let mut wb = calamine::Xlsx::new(f).unwrap();
    let meta: Vec<_> = wb.sheets_metadata().to_vec();
    for s in &meta {
        let h = wb.worksheet_range(&s.name).map(|r| r.height()).map_err(|e| format!("{e:?}"));
        println!("name={:?} typ={:?} height={:?}", s.name, s.typ, h);
    }
}
