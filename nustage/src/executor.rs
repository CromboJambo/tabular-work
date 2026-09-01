//! Executor — applies a `Pipeline` to a table, step by step.
//!
//! This is the "checkout": given a pipeline (the route) and a data file
//! (the terrain), replay the steps to produce the working table. Each step
//! runs on a `rsf::TypedTable`, so operations are type-aware.

use rsf::{Column, Expr, FieldValue, TypedTable};

use crate::pipeline::Step;
use crate::topology::TopologySignature;

/// Result of executing one step.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// The table after this step.
    pub table: TypedTable,
    /// Topology signature of the output table.
    pub topology: TopologySignature,
    /// Rows in before the step.
    pub rows_in: usize,
    /// Rows out after the step.
    pub rows_out: usize,
}

/// Error from executing a step.
#[derive(Debug, Clone, PartialEq)]
pub enum StepError {
    /// The referenced column does not exist in the current table.
    ColumnNotFound { column: String, available: Vec<String> },
    /// The expression could not be parsed.
    ExprParse(String),
    /// The expression is not a valid row predicate.
    InvalidPredicate(String),
    /// An aggregation function is not supported.
    UnknownAgg(String),
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepError::ColumnNotFound { column, available } => write!(
                f,
                "column '{}' not found (available: {})",
                column,
                available.join(", ")
            ),
            StepError::ExprParse(s) => write!(f, "expression parse error: {s}"),
            StepError::InvalidPredicate(s) => write!(f, "invalid predicate: {s}"),
            StepError::UnknownAgg(s) => write!(f, "unknown aggregation: {s}"),
        }
    }
}

impl std::error::Error for StepError {}

fn headers(table: &TypedTable) -> Vec<String> {
    table.columns.iter().map(|c| c.name.clone()).collect()
}

fn available(table: &TypedTable) -> Vec<String> {
    headers(table)
}

/// Apply a single step to a table.
pub fn apply_step(table: &TypedTable, step: &Step) -> Result<StepResult, StepError> {
    let rows_in = table.row_count();
    let out = match step {
        Step::FilterRows { condition } => filter_rows(table, condition)?,
        Step::AddColumn { name, expr } => add_column(table, name, expr)?,
        Step::GroupBy {
            group_cols,
            value_col,
            agg,
        } => group_by(table, group_cols, value_col, agg)?,
        Step::RenameColumn { from, to } => rename_column(table, from, to)?,
        Step::SelectColumns { columns } => select_columns(table, columns)?,
        Step::DropColumns { columns } => drop_columns(table, columns)?,
        Step::SortBy { column, desc } => sort_by(table, column, *desc)?,
        Step::RemoveDuplicates => remove_duplicates(table),
    };
    let rows_out = out.row_count();
    let topology = TopologySignature::derive(&headers(&out), &to_raw(&out));
    Ok(StepResult {
        table: out,
        topology,
        rows_in,
        rows_out,
    })
}

fn to_raw(table: &TypedTable) -> Vec<Vec<String>> {
    table.rows.iter().map(|r| r.iter().map(|v| v.as_str()).collect()).collect()
}

fn filter_rows(table: &TypedTable, condition: &str) -> Result<TypedTable, StepError> {
    let names = headers(table);
    validate_refs(condition, &names)?;
    let expr = Expr::parse(condition, &names)
        .map_err(|e| StepError::ExprParse(e.to_string()))?;
    Ok(table.where_clause(&expr))
}

/// Validate that every identifier in a condition/expression refers to a
/// column of the table.
///
/// The rsf parser degrades unknown identifiers to string literals instead
/// of erroring (so `amount > missing_col` parses fine and silently matches
/// nothing). This check catches that class of drift *before* execution,
/// which is what makes rebase conflicts loud instead of silent.
fn validate_refs(src: &str, names: &[String]) -> Result<(), StepError> {
    for id in extract_identifiers(src) {
        if !names.iter().any(|n| n.eq_ignore_ascii_case(&id)) {
            return Err(StepError::ColumnNotFound {
                column: id,
                available: names.to_vec(),
            });
        }
    }
    Ok(())
}

/// Extract bare identifiers from an expression source string, skipping
/// quoted string literals and SQL keywords.
fn extract_identifiers(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_str = false;
    let mut cur = String::new();
    for c in src.chars() {
        if in_str {
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            c if c.is_ascii_alphabetic() || c == '_' => cur.push(c),
            _ => {
                if !cur.is_empty() {
                    if !matches!(
                        cur.to_uppercase().as_str(),
                        "AND" | "OR" | "NOT" | "IS" | "IN" | "NULL"
                    ) {
                        out.push(cur.clone());
                    }
                    cur.clear();
                }
            }
        }
    }
    if !cur.is_empty()
        && !matches!(
            cur.to_uppercase().as_str(),
            "AND" | "OR" | "NOT" | "IS" | "IN" | "NULL"
        )
    {
        out.push(cur);
    }
    out
}

fn add_column(table: &TypedTable, name: &str, expr: &str) -> Result<TypedTable, StepError> {
    let names = headers(table);
    validate_refs(expr, &names)?;
    let e = Expr::parse(expr, &names).map_err(|e| StepError::ExprParse(e.to_string()))?;
    let mut out = table.clone();
    // Evaluate the expression per row; infer the new column's type from the
    // first non-null value.
    let mut inferred: Option<rsf::TypeHint> = None;
    let mut new_rows: Vec<Vec<FieldValue>> = Vec::with_capacity(table.rows.len());
    for row in &table.rows {
        let v = evaluate_value(&e, row, &table.columns);
        if inferred.is_none() && !v.is_null() {
            inferred = Some(infer_hint(&v));
        }
        let mut nr = row.clone();
        nr.push(v);
        new_rows.push(nr);
    }
    let type_hint = inferred.unwrap_or_else(|| rsf::TypeHint::Unknown);
    out.columns.push(Column::new(name, type_hint));
    out.rows = new_rows;
    Ok(out)
}

/// Evaluate an expression to a single FieldValue (not a boolean).
///
/// Arithmetic operators (Add/Sub/Mul/Div) produce a numeric value via
/// `FieldValue::apply_arith`; comparison operators (Eq/Neq/Gt/Lt/Ge/Le)
/// in value context yield a Boolean.
fn evaluate_value(expr: &Expr, row: &[FieldValue], columns: &[Column]) -> FieldValue {
    use rsf::Expr::*;
    match expr {
        Column(name) => {
            if let Some(idx) = columns.iter().position(|c| &c.name == name) {
                row.get(idx).cloned().unwrap_or(FieldValue::Null)
            } else {
                FieldValue::Null
            }
        }
        Literal(v) => v.clone(),
        Negate(inner) => {
            let v = evaluate_value(inner, row, columns);
            match FieldValue::as_f64(&v) {
                Some(n) if n.fract() == 0.0 && n.abs() < i64::MAX as f64 => {
                    FieldValue::Integer(-(n as i64))
                }
                Some(n) => FieldValue::Float(-n),
                None => FieldValue::Null,
            }
        }
        BinaryOp(l, op, r) => {
            let lv = evaluate_value(l, row, columns);
            let rv = evaluate_value(r, row, columns);
            match op {
                rsf::BinOp::Add | rsf::BinOp::Sub | rsf::BinOp::Mul | rsf::BinOp::Div => {
                    FieldValue::apply_arith(&lv, &rv, op)
                }
                _ => FieldValue::Boolean(compare_values(&lv, &rv, op)),
            }
        }
        LogicalOp(_, l, r) => FieldValue::Boolean(l.evaluate(row, columns) && r.evaluate(row, columns)),
        Not(inner) => FieldValue::Boolean(!inner.evaluate(row, columns)),
        NullCheck(name, is_not) => {
            if let Some(idx) = columns.iter().position(|c| &c.name == name) {
                let is_null = row.get(idx).map_or(true, |v| v.is_null());
                FieldValue::Boolean(*is_not != is_null)
            } else {
                FieldValue::Null
            }
        }
        In(name, values) => {
            if let Some(idx) = columns.iter().position(|c| &c.name == name) {
                let v = row.get(idx).cloned().unwrap_or(FieldValue::Null);
                FieldValue::Boolean(values.contains(&v))
            } else {
                FieldValue::Null
            }
        }
    }
}

/// Compare two field values with a binary operator (mirrors rsf's semantics).
fn compare_values(l: &FieldValue, r: &FieldValue, op: &rsf::BinOp) -> bool {
    use rsf::BinOp::*;
    match (l, r) {
        (FieldValue::Null, FieldValue::Null) => matches!(op, Eq),
        (FieldValue::Null, _) | (_, FieldValue::Null) => matches!(op, Neq),
        _ => {
            if let (Some(a), Some(b)) = (as_f64(l), as_f64(r)) {
                return match op {
                    Eq => (a - b).abs() < 1e-9,
                    Neq => (a - b).abs() >= 1e-9,
                    Gt => a > b,
                    Lt => a < b,
                    Ge => a >= b,
                    Le => a <= b,
                    // Arithmetic ops are routed to apply_arith by evaluate_value
                    // before reaching here; they never arrive as a comparison.
                    Add | Sub | Mul | Div => false,
                };
            }
            let ls = l.as_str();
            let rs = r.as_str();
            match op {
                Eq => ls == rs,
                Neq => ls != rs,
                Gt => ls > rs,
                Lt => ls < rs,
                Ge => ls >= rs,
                Le => ls <= rs,
                Add | Sub | Mul | Div => false,
            }
        }
    }
}

fn as_f64(v: &FieldValue) -> Option<f64> {
    match v {
        FieldValue::Integer(n) => Some(*n as f64),
        FieldValue::Float(f) => Some(*f),
        FieldValue::Text(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn infer_hint(v: &FieldValue) -> rsf::TypeHint {
    match v {
        FieldValue::Null => rsf::TypeHint::Unknown,
        FieldValue::Integer(_) => rsf::TypeHint::Integer,
        FieldValue::Float(_) => rsf::TypeHint::Float,
        FieldValue::Boolean(_) => rsf::TypeHint::Boolean,
        FieldValue::Text(_) => rsf::TypeHint::Unknown,
    }
}

fn group_by(
    table: &TypedTable,
    group_cols: &[String],
    value_col: &str,
    agg: &str,
) -> Result<TypedTable, StepError> {
    // Validate group columns exist.
    let names = headers(table);
    for gc in group_cols {
        if !names.contains(gc) {
            return Err(StepError::ColumnNotFound {
                column: gc.clone(),
                available: available(table),
            });
        }
    }
    if !names.contains(&value_col.to_string()) {
        return Err(StepError::ColumnNotFound {
            column: value_col.to_string(),
            available: available(table),
        });
    }
    let agg = agg.to_lowercase();
    if !matches!(agg.as_str(), "sum" | "count" | "min" | "max" | "avg") {
        return Err(StepError::UnknownAgg(agg));
    }

    // Group rows by the concatenated group-column string, preserving first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (i, row) in table.rows.iter().enumerate() {
        let key: String = group_cols
            .iter()
            .map(|gc| {
                let idx = table.find_column(gc).unwrap();
                row.get(idx).map(|v| v.as_str()).unwrap_or_else(|| "NULL".to_string())
            })
            .collect::<Vec<_>>()
            .join("\x1f");
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(i);
    }

    let value_idx = table.find_column(value_col).unwrap();
    let mut out = TypedTable::new();
    for gc in group_cols {
        let src = table.find_column(gc).unwrap();
        out.columns.push(table.columns[src].clone());
    }
    let agg_name = format!("{value_col}_{agg}");
    out.columns.push(Column::new(agg_name, rsf::TypeHint::Float));

    for key in &order {
        let indices = &groups[key];
        let mut out_row: Vec<FieldValue> = Vec::new();
        for gc in group_cols {
            let idx = table.find_column(gc).unwrap();
            out_row.push(table.rows[indices[0]].get(idx).cloned().unwrap_or(FieldValue::Null));
        }
        let vals: Vec<f64> = indices
            .iter()
            .filter_map(|&i| {
                table.rows[i]
                    .get(value_idx)
                    .and_then(|v| as_f64(v))
            })
            .collect();
        let agg_val = match agg.as_str() {
            "count" => indices.len() as f64,
            "sum" => vals.iter().sum::<f64>(),
            "min" => vals.iter().cloned().fold(f64::INFINITY, f64::min),
            "max" => vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            "avg" => {
                if vals.is_empty() {
                    f64::NAN
                } else {
                    vals.iter().sum::<f64>() / vals.len() as f64
                }
            }
            _ => unreachable!(),
        };
        out_row.push(FieldValue::Float(agg_val));
        out.rows.push(out_row);
    }
    Ok(out)
}

fn rename_column(table: &TypedTable, from: &str, to: &str) -> Result<TypedTable, StepError> {
    let mut out = table.clone();
    match out.columns.iter_mut().find(|c| c.name == from) {
        Some(c) => c.name = to.to_string(),
        None => {
            return Err(StepError::ColumnNotFound {
                column: from.to_string(),
                available: available(table),
            })
        }
    }
    Ok(out)
}

fn select_columns(table: &TypedTable, columns: &[String]) -> Result<TypedTable, StepError> {
    let names = headers(table);
    let indices: Vec<usize> = columns
        .iter()
        .map(|c| {
            table.find_column(c).ok_or_else(|| StepError::ColumnNotFound {
                column: c.clone(),
                available: names.clone(),
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(table.select_columns(&indices))
}

fn drop_columns(table: &TypedTable, columns: &[String]) -> Result<TypedTable, StepError> {
    let names = headers(table);
    let drop: Vec<usize> = columns
        .iter()
        .map(|c| {
            table.find_column(c).ok_or_else(|| StepError::ColumnNotFound {
                column: c.clone(),
                available: names.clone(),
            })
        })
        .collect::<Result<_, _>>()?;
    let keep: Vec<usize> = (0..table.column_count())
        .filter(|i| !drop.contains(i))
        .collect();
    Ok(table.select_columns(&keep))
}

fn sort_by(table: &TypedTable, column: &str, desc: bool) -> Result<TypedTable, StepError> {
    let idx = table
        .find_column(column)
        .ok_or_else(|| StepError::ColumnNotFound {
            column: column.to_string(),
            available: available(table),
        })?;
    let mut out = table.clone();
    out.rows.sort_by(|a, b| {
        let av = a.get(idx).map(FieldValue::as_str);
        let bv = b.get(idx).map(FieldValue::as_str);
        // Numeric-aware compare when both parse as f64.
        match (av.as_ref().and_then(|s| s.parse::<f64>().ok()), bv.as_ref().and_then(|s| s.parse::<f64>().ok())) {
            (Some(an), Some(bn)) => an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal),
            _ => av.cmp(&bv),
        }
    });
    if desc {
        out.rows.reverse();
    }
    Ok(out)
}

fn remove_duplicates(table: &TypedTable) -> TypedTable {
    let mut seen: std::collections::HashSet<Vec<String>> = std::collections::HashSet::new();
    let mut out = TypedTable::new();
    out.columns = table.columns.clone();
    for row in &table.rows {
        let key: Vec<String> = row.iter().map(|v| v.as_str()).collect();
        if seen.insert(key) {
            out.rows.push(row.clone());
        }
    }
    out
}

/// Execute a full pipeline over an initial table, returning the final table
/// and the per-step ledger.
pub fn run_pipeline(
    table: &TypedTable,
    steps: &[Step],
) -> Result<(TypedTable, Vec<StepResult>), (usize, StepError)> {
    let mut current = table.clone();
    let mut ledger = Vec::with_capacity(steps.len());
    for (i, step) in steps.iter().enumerate() {
        match apply_step(&current, step) {
            Ok(res) => {
                current = res.table.clone();
                ledger.push(res);
            }
            Err(e) => return Err((i, e)),
        }
    }
    Ok((current, ledger))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Step;
    use crate::rebase::build_typed;

    /// Three order rows: qty * price = 200, 150, 250.
    fn order_table() -> TypedTable {
        let headers = vec![
            "id".to_string(),
            "qty".to_string(),
            "price".to_string(),
        ];
        let rows = vec![
            vec!["1".to_string(), "2".to_string(), "100".to_string()],
            vec!["2".to_string(), "3".to_string(), "50".to_string()],
            vec!["3".to_string(), "1".to_string(), "250".to_string()],
        ];
        build_typed(&headers, &rows)
    }

    fn last_col(out: &TypedTable) -> Vec<&FieldValue> {
        let idx = out.column_count() - 1;
        out.rows.iter().map(|r| &r[idx]).collect()
    }

    /// The canonical case: `qty * price` must produce real derived values,
    /// not nulls or text, through the full public pipeline path.
    #[test]
    fn test_add_column_derives_real_values() {
        let table = order_table();
        let steps = vec![Step::AddColumn {
            name: "total".to_string(),
            expr: "qty * price".to_string(),
        }];
        let (out, _ledger) = run_pipeline(&table, &steps).expect("pipeline should run");
        assert_eq!(out.column_count(), 4);
        assert_eq!(out.columns[3].name, "total");
        let totals = last_col(&out);
        assert_eq!(totals[0], &FieldValue::Integer(200));
        assert_eq!(totals[1], &FieldValue::Integer(150));
        assert_eq!(totals[2], &FieldValue::Integer(250));
    }

    /// Unary minus in a value expression: `-qty` negates the column.
    #[test]
    fn test_add_column_unary_minus() {
        let table = order_table();
        let steps = vec![Step::AddColumn {
            name: "neg".to_string(),
            expr: "-qty".to_string(),
        }];
        let (out, _) = run_pipeline(&table, &steps).expect("pipeline should run");
        let negs = last_col(&out);
        assert_eq!(negs[0], &FieldValue::Integer(-2));
        assert_eq!(negs[1], &FieldValue::Integer(-3));
        assert_eq!(negs[2], &FieldValue::Integer(-1));
    }

    /// Binary minus in a value expression: `qty - price` subtracts.
    #[test]
    fn test_add_column_binary_minus() {
        let table = order_table();
        let steps = vec![Step::AddColumn {
            name: "diff".to_string(),
            expr: "qty - price".to_string(),
        }];
        let (out, _) = run_pipeline(&table, &steps).expect("pipeline should run");
        let diffs = last_col(&out);
        // 2 - 100 = -98, 3 - 50 = -47, 1 - 250 = -249
        assert_eq!(diffs[0], &FieldValue::Integer(-98));
        assert_eq!(diffs[1], &FieldValue::Integer(-47));
        assert_eq!(diffs[2], &FieldValue::Integer(-249));
    }

    /// A derived arithmetic column can feed a later filter step.
    #[test]
    fn test_add_column_then_filter() {
        let table = order_table();
        let steps = vec![
            Step::AddColumn {
                name: "total".to_string(),
                expr: "qty * price".to_string(),
            },
            Step::FilterRows {
                condition: "total > 150".to_string(),
            },
        ];
        let (out, _) = run_pipeline(&table, &steps).expect("pipeline should run");
        // Only rows with total 200 and 250 survive (150 is not > 150).
        assert_eq!(out.row_count(), 2);
    }
}
