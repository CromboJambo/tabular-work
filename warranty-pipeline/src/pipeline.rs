//! Pipeline transformation operations (final fixed version)

use crate::io::{load_csv, save_csv};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn load_csv(path: &str) -> Result<Self> {
        let raw_rows = crate::io::load_csv(path)?;
        if raw_rows.is_empty() {
            return Err(anyhow::anyhow!("CSV file is empty"));
        }

        Ok(Self {
            headers: raw_rows[0].clone(),
            rows: raw_rows[1..].to_vec(),
        })
    }

    pub fn from_data(data: Vec<Vec<String>>) -> Result<Self> {
        if data.is_empty() {
            return Err(anyhow::anyhow!("Data is empty"));
        }
        let mut iter = data.iter();
        let headers = iter.next().unwrap().clone();
        let rows: Vec<Vec<String>> = iter.cloned().collect();

        Ok(Self { headers, rows })
    }

    pub fn save_csv(&self, path: &str) -> Result<()> {
        let mut all_rows = vec![self.headers.clone()];
        all_rows.extend(self.rows.iter().cloned());
        save_csv(&all_rows, path)
    }

    pub fn filter(&self, condition: &str) -> Result<Self> {
        let parts: Vec<&str> = condition.split(" && ").collect();
        let filtered: Vec<Vec<String>> = self
            .rows
            .iter()
            .filter(|row| parts.iter().all(|p| self.eval_cond(row, p)))
            .cloned()
            .collect();
        Ok(Self {
            headers: self.headers.clone(),
            rows: filtered,
        })
    }

    fn eval_cond(&self, row: &[String], cond: &str) -> bool {
        let mut parts = cond.split_whitespace();
        if let (Some(col), Some(op), Some(val)) = (parts.next(), parts.next(), parts.next()) {
            if let Some(idx) = self.headers.iter().position(|h| h == col) {
                let actual = row.get(idx).map(|s| s.as_str()).unwrap_or("");
                let val = val.trim_matches('\'');
                return match op {
                    "==" => actual == val,
                    "!=" => actual != val,
                    _ => false,
                };
            }
        }
        false
    }

    fn col_idx(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }

    pub fn pivot(&self, index_cols: &[&str], value_col: &str, agg: &str) -> Result<Self> {
        let idxs: Vec<usize> = index_cols
            .iter()
            .map(|c| {
                self.col_idx(c)
                    .ok_or_else(|| anyhow::anyhow!("Col {} not found", c))
            })
            .collect::<Result<Vec<_>>>()?;
        let val_idx = self
            .col_idx(value_col)
            .ok_or_else(|| anyhow::anyhow!("Val col {} not found", value_col))?;

        let mut groups: HashMap<String, Vec<&Vec<String>>> = HashMap::new();
        for row in &self.rows {
            let key: String = idxs
                .iter()
                .map(|i| row[*i].clone())
                .collect::<Vec<_>>()
                .join("__X__");
            groups.entry(key).or_default().push(row);
        }

        let mut out_headers: Vec<String> = index_cols.iter().map(|s| s.to_string()).collect();
        out_headers.push(format!("{}_{}", value_col, agg));

        let mut out_rows: Vec<Vec<String>> = Vec::new();
        for (_k, rows) in groups {
            let vals: Vec<f64> = rows
                .iter()
                .filter_map(|r| r[val_idx].parse().ok())
                .collect();
            let agg_val = match agg {
                "sum" => vals.iter().sum::<f64>().to_string(),
                "count" => vals.len().to_string(),
                _ => "0".to_string(),
            };
            let mut row: Vec<String> = idxs.iter().map(|i| rows[0][*i].clone()).collect();
            row.push(agg_val);
            out_rows.push(row);
        }
        Ok(Self {
            headers: out_headers,
            rows: out_rows,
        })
    }

    pub fn join(&self, other: &Table, on: &[&str], _ty: &str) -> Result<Self> {
        let left_idx: Vec<usize> = on
            .iter()
            .map(|c| {
                self.col_idx(c)
                    .ok_or_else(|| anyhow::anyhow!("Left {} not found", c))
            })
            .collect::<Result<Vec<_>>>()?;
        let right_idx: Vec<usize> = on
            .iter()
            .map(|c| {
                other
                    .col_idx(c)
                    .ok_or_else(|| anyhow::anyhow!("Right {} not found", c))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut idx: HashMap<String, &Vec<String>> = HashMap::new();
        for row in &other.rows {
            let key: String = right_idx
                .iter()
                .map(|i| row[*i].clone())
                .collect::<Vec<_>>()
                .join("__X__");
            idx.insert(key, row);
        }

        // Build output headers: left headers + non-join right headers
        let mut out_headers = self.headers.clone();
        for (i, h) in other.headers.iter().enumerate() {
            if !right_idx.contains(&i) {
                out_headers.push(h.clone());
            }
        }

        let mut out_rows: Vec<Vec<String>> = Vec::new();
        for lr in &self.rows {
            let key: String = left_idx
                .iter()
                .map(|i| lr[*i].clone())
                .collect::<Vec<_>>()
                .join("__X__");
            if let Some(rr) = idx.get(&key) {
                let mut merged = lr.clone();
                // Add only non-join columns from right table
                for (i, val) in rr.iter().enumerate() {
                    if !right_idx.contains(&i) {
                        merged.push(val.clone());
                    }
                }
                out_rows.push(merged);
            }
        }
        Ok(Self {
            headers: out_headers,
            rows: out_rows,
        })
    }

    pub fn select(&self, cols: &[&str]) -> Result<Self> {
        let idxs: Vec<usize> = cols
            .iter()
            .map(|c| {
                self.col_idx(c)
                    .ok_or_else(|| anyhow::anyhow!("Col {} not found", c))
            })
            .collect::<Result<Vec<_>>>()?;
        let headers: Vec<String> = idxs.iter().map(|i| self.headers[*i].clone()).collect();
        let rows: Vec<Vec<String>> = self
            .rows
            .iter()
            .map(|r| idxs.iter().map(|i| r[*i].clone()).collect())
            .collect();
        Ok(Self { headers, rows })
    }

    pub fn mutate(&self, defs: &[(&str, &str)]) -> Result<Self> {
        let mut nh = self.headers.clone();
        let mut nr = self.rows.clone();
        for (name, expr) in defs {
            nh.push(name.to_string());
            for row in &mut nr {
                row.push(self.eval_expr(row, expr));
            }
        }
        Ok(Self {
            headers: nh,
            rows: nr,
        })
    }

    fn eval_expr(&self, row: &[String], expr: &str) -> String {
        if let Some(c) = expr
            .strip_prefix("UPPER(")
            .and_then(|s| s.strip_suffix(')'))
        {
            if let Some(i) = self.col_idx(c) {
                return row[i].to_uppercase();
            }
        }
        if let [a, op, b] = expr.split_whitespace().collect::<Vec<_>>().as_slice() {
            if let (Some(i1), Some(i2)) = (self.col_idx(a), self.col_idx(b)) {
                let v1: f64 = row[i1].parse().unwrap_or(0.0);
                let v2: f64 = row[i2].parse().unwrap_or(0.0);
                return match *op {
                    "*" => (v1 * v2).to_string(),
                    "+" => (v1 + v2).to_string(),
                    _ => "0".to_string(),
                };
            }
        }
        String::new()
    }

    pub fn sort(&self, cols: &[(&str, bool)]) -> Result<Self> {
        let idxs: Vec<usize> = cols
            .iter()
            .map(|(c, _)| {
                self.col_idx(c)
                    .ok_or_else(|| anyhow::anyhow!("Col {} not found", c))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut indexed: Vec<(usize, Vec<String>)> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, r)| (i, r.clone()))
            .collect();

        // Sort using indices properly - zip gives us (&usize, &(&str, bool))
        indexed.sort_by(|a, b| {
            for (idx, (_name, desc)) in idxs.iter().zip(cols) {
                let va = a.1[*idx].clone();
                let vb = b.1[*idx].clone();
                let c = va.cmp(&vb);
                if *desc && c != std::cmp::Ordering::Equal {
                    return c.reverse();
                }
                if !*desc && c != std::cmp::Ordering::Equal {
                    return c;
                }
            }
            std::cmp::Ordering::Equal
        });

        let rows: Vec<Vec<String>> = indexed.iter().map(|(_, r)| r.clone()).collect();
        Ok(Self {
            headers: self.headers.clone(),
            rows,
        })
    }

    pub fn group_by(&self, group_cols: &[&str], aggregations: &[(&str, &str)]) -> Result<Self> {
        let idxs: Vec<usize> = group_cols
            .iter()
            .map(|c| {
                self.col_idx(c)
                    .ok_or_else(|| anyhow::anyhow!("Group col {} not found", c))
            })
            .collect::<Result<Vec<_>>>()?;

        let val_idxs: Vec<usize> = aggregations
            .iter()
            .map(|(col, _)| {
                self.col_idx(col)
                    .ok_or_else(|| anyhow::anyhow!("Agg col {} not found", col))
            })
            .collect::<Result<Vec<_>>>()?;

        // Group rows by the group columns
        let mut groups: HashMap<String, Vec<&Vec<String>>> = HashMap::new();
        for row in &self.rows {
            let key: String = idxs
                .iter()
                .map(|i| row[*i].clone())
                .collect::<Vec<_>>()
                .join("__X__");
            groups.entry(key).or_default().push(row);
        }

        // Build output headers
        let mut out_headers: Vec<String> = group_cols.iter().map(|s| s.to_string()).collect();
        for (col, agg) in aggregations {
            out_headers.push(format!("{}_{}", col, agg));
        }

        // Compute aggregates for each group
        let mut out_rows: Vec<Vec<String>> = Vec::new();
        for (_k, rows) in groups {
            let mut row: Vec<String> = idxs.iter().map(|i| rows[0][*i].clone()).collect();

            for (idx, (_col, agg)) in val_idxs.iter().zip(aggregations) {
                let agg_val = match *agg {
                    "sum" => {
                        let vals: Vec<f64> =
                            rows.iter().filter_map(|r| r[*idx].parse().ok()).collect();
                        vals.iter().sum::<f64>().to_string()
                    }
                    "count" => rows.len().to_string(),
                    "avg" => {
                        let vals: Vec<f64> =
                            rows.iter().filter_map(|r| r[*idx].parse().ok()).collect();
                        if vals.is_empty() {
                            "0".to_string()
                        } else {
                            (vals.iter().sum::<f64>() / vals.len() as f64).to_string()
                        }
                    }
                    "min" => {
                        let vals: Vec<f64> =
                            rows.iter().filter_map(|r| r[*idx].parse().ok()).collect();
                        vals.iter()
                            .cloned()
                            .fold(f64::INFINITY, f64::min)
                            .to_string()
                    }
                    "max" => {
                        let vals: Vec<f64> =
                            rows.iter().filter_map(|r| r[*idx].parse().ok()).collect();
                        vals.iter()
                            .cloned()
                            .fold(f64::NEG_INFINITY, f64::max)
                            .to_string()
                    }
                    _ => "0".to_string(),
                };
                row.push(agg_val);
            }

            out_rows.push(row);
        }

        Ok(Self {
            headers: out_headers,
            rows: out_rows,
        })
    }
}
