//! sro-pivot: reproducible group-by/sum pivot for Syteline "ToExcel" exports.
//!
//! Extracts the *transformations* from a workbook like
//! `ToExcel_ServiceOrderTransactions_LABOR_ACCR_REC.xlsx`:
//!   - read the main sheet (header row 1, data rows 2..n)
//!   - drop Excel's auto-generated table totals row (last row of the table)
//!   - optional filter (e.g. SRO = SRO0003695) — mirrors the pivot's field filter
//!   - group by the key columns, summing the measure columns (first-seen order preserved)
//!   - emit CSV with a Grand Total row

use anyhow::{Context, Result};
use calamine::Reader;
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

#[derive(Parser)]
#[command(
    name = "sro-pivot",
    about = "Reproducible pivot for Syteline ToExcel exports"
)]
struct Args {
    /// Path to the .xlsx workbook
    xlsx: String,

    /// Sheet name (default: first sheet)
    #[arg(long)]
    sheet: Option<String>,

    /// Filter: column=value (exact), or column<value / column<=value /
    /// column>value / column>=value (dates compare as ISO dates, numbers as f64).
    /// Repeatable; all must match. Mirrors a pivot field filter/slicer.
    #[arg(long = "filter")]
    filters: Vec<String>,

    /// Key columns for group-by, in output order (comma-separated)
    #[arg(
        long,
        default_value = "SRO,SRO Description,Line Description,Oper Description,Billing Code,Work Code Description,Partner,Invoice,Trans Date"
    )]
    keys: String,

    /// Measure columns to sum, in output order (comma-separated)
    #[arg(long, default_value = "Hours Worked,Hours To Bill,Ext Cost,Ext Price")]
    measures: String,

    /// Sort key columns for the output rows, comma-separated.
    /// Values are sorted naturally: numeric-looking values as numbers, else strings.
    /// (Excel's pivot sorts row fields in cache order; this mirrors a manual sort.)
    #[arg(long)]
    sort: Option<String>,

    /// Output CSV path (default: stdout)
    #[arg(short, long)]
    out: Option<String>,
}

/// Cell value as read from the sheet.
#[derive(Debug, Clone)]
enum Val {
    Str(String),
    Num(f64),
    Date(chrono::NaiveDate),
    Empty,
}

impl Val {
    fn display(&self) -> String {
        match self {
            Val::Str(s) => s.clone(),
            Val::Num(n) => format_num(*n),
            Val::Date(d) => d.format("%Y-%m-%d").to_string(),
            Val::Empty => String::new(),
        }
    }
}

/// Format a number the way Excel shows it: integers without a decimal point,
/// and float-summation noise (e.g. 99881.76000000001) trimmed away.
fn format_num(n: f64) -> String {
    let n = (n * 1e10).round() / 1e10; // kill float summation noise
    if n == n.trunc() && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    let s = format!("{:.6}", n);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

fn data_to_val(d: calamine::Data) -> Val {
    match d {
        calamine::Data::Empty => Val::Empty,
        calamine::Data::String(s) => Val::Str(s),
        calamine::Data::Float(f) => Val::Num(f),
        calamine::Data::Int(i) => Val::Num(i as f64),
        calamine::Data::Bool(b) => Val::Str(if b { "1" } else { "0" }.to_string()),
        calamine::Data::Error(e) => Val::Str(format!("{:?}", e)),
        // Calamine decodes date-formatted cells into ExcelDateTime (serial + type).
        calamine::Data::DateTime(dt) => match dt.as_datetime() {
            Some(dt) => Val::Date(dt.date()),
            None => Val::Num(dt.as_f64()),
        },
        calamine::Data::DateTimeIso(s) => Val::Str(s),
        calamine::Data::DurationIso(s) => Val::Str(s),
    }
}

/// Read a sheet into header + data rows.
fn read_sheet(path: &Path, sheet_name: Option<&str>) -> Result<(Vec<String>, Vec<Vec<Val>>)> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut wb = calamine::Xlsx::new(file).context("parse xlsx")?;

    // Pick the sheet: by name, or the first *non-empty* visible worksheet
    // (Syteline ToExcel exports lead with an empty "data_source_howto" sheet).
    let meta: Vec<_> = wb.sheets_metadata().to_vec();
    for s in &meta {
        let h = wb.worksheet_range(&s.name).map(|r| (r.height(), r.width())).map_err(|e| format!("{e:?}"));
        eprintln!("DBG sheet={:?} typ={:?} range={:?}", s.name, s.typ, h);
    }
    let name = match sheet_name {
        Some(n) => n.to_string(),
        None => meta
            .iter()
            .find(|s| s.typ == calamine::SheetType::WorkSheet)
            .filter(|s| !wb.worksheet_range(&s.name).map(|r| r.height() == 0).unwrap_or(true))
            .or_else(|| meta.iter().find(|s| s.typ == calamine::SheetType::WorkSheet))
            .or_else(|| meta.first())
            .map(|s| s.name.clone())
            .unwrap_or_default(),
    };

    let range = wb
        .worksheet_range(&name)
        .with_context(|| format!("sheet '{}'", name))?;

    let mut rows: Vec<Vec<Val>> = Vec::new();
    for r in 0..range.height() {
        let row: Vec<Val> = (0..range.width())
            .map(|c| range.get((r, c)).cloned().unwrap_or_default())
            .map(data_to_val)
            .collect();
        rows.push(row);
    }

    if rows.is_empty() {
        anyhow::bail!("sheet '{}' is empty", name);
    }

    let headers: Vec<String> = rows[0].iter().map(|v| v.display()).collect();
    Ok((headers, rows[1..].to_vec()))
}

fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let keys: Vec<&str> = args
        .keys
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let measures: Vec<&str> = args
        .measures
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Parse filters: col=value | col<value | col<=value | col>value | col>=value
    #[derive(Clone, Copy, PartialEq)]
    enum Op { Eq, Lt, Le, Gt, Ge }
    struct Filter { col: String, op: Op, val: String }

    let mut filters: Vec<Filter> = Vec::new();
    for f in &args.filters {
        let (col, op, val) = if let Some((c, v)) = f.split_once("<=") {
            (c.trim(), Op::Le, v.trim())
        } else if let Some((c, v)) = f.split_once(">=") {
            (c.trim(), Op::Ge, v.trim())
        } else if let Some((c, v)) = f.split_once('<') {
            (c.trim(), Op::Lt, v.trim())
        } else if let Some((c, v)) = f.split_once('>') {
            (c.trim(), Op::Gt, v.trim())
        } else if let Some((c, v)) = f.split_once('=') {
            (c.trim(), Op::Eq, v.trim())
        } else {
            anyhow::bail!("bad filter '{}'", f);
        };
        filters.push(Filter { col: col.to_string(), op, val: val.to_string() });
    }

    /// Compare a cell value against a filter operand. Dates compare as ISO strings;
    /// numeric-looking values compare numerically; else string comparison.
    fn cell_matches(cell: &Val, op: Op, target: &str) -> bool {
        let cmp = match (cell, target.parse::<f64>().ok()) {
            (Val::Num(n), Some(t)) => n.partial_cmp(&t),
            (Val::Date(d), _) => Some(d.format("%Y-%m-%d").to_string().as_str().cmp(target)),
            (Val::Str(s), _) => Some(s.as_str().cmp(target)),
            _ => None,
        };
        match (cmp, op) {
            (Some(std::cmp::Ordering::Equal), Op::Eq) => true,
            (Some(std::cmp::Ordering::Less), Op::Lt | Op::Le) => true,
            (Some(std::cmp::Ordering::Greater), Op::Gt | Op::Ge) => true,
            _ => false,
        }
    }

    let (headers, mut data_rows) = read_sheet(Path::new(&args.xlsx), args.sheet.as_deref())?;
    let idx: HashMap<&str, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.as_str(), i))
        .collect();

    for k in keys.iter().chain(measures.iter()) {
        if !idx.contains_key(k) {
            anyhow::bail!("column '{}' not found; have: {}", k, headers.join(", "));
        }
    }

    // Drop the Excel table totals row: last row where all keys are empty but a measure is set.
    if let Some(last) = data_rows.last() {
        let keys_empty = keys.iter().all(|k| matches!(last[idx[k]], Val::Empty));
        let measures_nonempty = measures.iter().any(|m| !matches!(last[idx[m]], Val::Empty));
        if keys_empty && measures_nonempty {
            eprintln!("dropping table totals row (row {})", data_rows.len() + 1);
            data_rows.pop();
        }
    }

    let n_total = data_rows.len();

    // Apply filters
    let filtered: Vec<&Vec<Val>> = data_rows
        .iter()
        .filter(|row| {
            filters.iter().all(|f| {
                idx.get(f.col.as_str()).map(|&ci| cell_matches(&row[ci], f.op, &f.val)).unwrap_or(false)
            })
        })
        .collect();

    eprintln!("rows: {} total, {} after filter", n_total, filtered.len());

    // Group by keys (first-seen order), summing measures.
    struct Agg {
        sums: Vec<f64>,
    }
    let mut groups: Vec<(Vec<Val>, Agg)> = Vec::new();
    let mut index: HashMap<Vec<String>, usize> = HashMap::new();

    for row in &filtered {
        let key_vals: Vec<Val> = keys.iter().map(|k| row[idx[k]].clone()).collect();
        let key_strs: Vec<String> = key_vals.iter().map(|v| v.display()).collect();
        let pos = match index.get(&key_strs) {
            Some(&p) => p,
            None => {
                groups.push((key_vals.clone(), Agg { sums: vec![0.0; measures.len()] }));
                index.insert(key_strs, groups.len() - 1);
                groups.len() - 1
            }
        };
        for (i, m) in measures.iter().enumerate() {
            if let Val::Num(n) = &row[idx[m]] {
                groups[pos].1.sums[i] += *n;
            }
        }
    }

    // Render output.
    let mut out_rows: Vec<Vec<String>> = Vec::new();
    out_rows.push(
        [keys.clone(), measures.clone()]
            .concat()
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );

    // Optional sort on key columns (natural: numeric-looking values as numbers).
    // Spec: comma-separated key names, each optionally suffixed ":desc".
    if let Some(sort_spec) = &args.sort {
        let mut sort_cols: Vec<(usize, bool)> = Vec::new();
        for c in sort_spec.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let (name, desc) = match c.split_once(":desc") {
                Some((n, _)) => (n, true),
                None => (c, false),
            };
            let pos = keys
                .iter()
                .position(|k| *k == name)
                .with_context(|| format!("sort column '{}' not in keys", name))?;
            sort_cols.push((pos, desc));
        }

        groups.sort_by(|a, b| {
            for &(ci, desc) in &sort_cols {
                let ord = match (&a.0[ci], &b.0[ci]) {
                    (Val::Num(x), Val::Num(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                    (x, y) => x.display().cmp(&y.display()),
                };
                let ord = if desc { ord.reverse() } else { ord };
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        });
    }

    for (key_vals, agg) in &groups {
        let mut row: Vec<String> = key_vals.iter().map(|v| v.display()).collect();
        row.extend(agg.sums.iter().map(|n| format_num(*n)));
        out_rows.push(row);
    }

    // Grand total row
    let mut gt: Vec<String> = vec!["Grand Total".to_string()];
    for _ in 1..keys.len() {
        gt.push(String::new());
    }
    let totals: Vec<f64> = (0..measures.len())
        .map(|i| groups.iter().map(|g| g.1.sums[i]).sum::<f64>())
        .collect();
    gt.extend(totals.iter().map(|n| format_num(*n)));
    out_rows.push(gt);

    // Write CSV or stdout
    if let Some(out_path) = &args.out {
        let file = File::create(out_path)?;
        let mut w = csv::Writer::from_writer(file);
        for row in &out_rows {
            w.write_record(row)?;
        }
        w.flush()?;
        eprintln!("wrote {} rows to {}", out_rows.len(), out_path);
    } else {
        for row in &out_rows {
            println!("{}", row.iter().map(|f| csv_field(f)).collect::<Vec<_>>().join(","));
        }
    }

    Ok(())
}
