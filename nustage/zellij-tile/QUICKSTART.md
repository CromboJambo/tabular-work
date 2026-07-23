# Nustage Zellij Tile - Quick Start Guide

Get up and running with the Nustage Zellij tile in 5 minutes.

## What is This?

A Zellij tile that provides data exploration and transformation capabilities using Nustage's transformation engine. Think of it as Power Query for the terminal.

## Installation

### 1. Install Zellij

```bash
cargo install zellij
```

### 2. Build the Nustage Tile

```bash
cd nustage/zellij-tile
cargo build --release
```

## Quick Start

### Step 1: Start Zellij

```bash
zellij -S mysession
```

### Step 2: Add the Tile

1. Press `Ctrl+Shift+I` to open the tile list
2. Type `nustage` and select it
3. Press `Enter`

### Step 3: Load Data

1. Press `l` in the tile
2. Navigate to your data file (CSV, Parquet, JSON, or TSV)
3. Press `Enter`

### Step 4: Add Transformations

1. Press `a` to add a new transformation
2. Select the transformation type
3. Press `Enter`

### Step 5: View Results

- `s` - View steps
- `d` - View data
- `z` - View schema
- `x` - View SQL

## Basic Workflows

### Simple Filter

```bash
# Load data
l

# Add filter
a → Filter Rows

# View data
d
```

### Group and Aggregate

```bash
# Load data
l

# Add group by
a → Group By

# Add aggregation
a → Sum

# View results
d
```

### Export M Code

```bash
# Build transformations
# Add steps as needed

# View SQL
x

# Copy the SQL for use in Excel or Power Query
```

## Keybindings

| Key | Action |
|-----|--------|
| `l` | Load data file |
| `s` | Switch to Steps view |
| `d` | Switch to Data view |
| `z` | Switch to Schema view |
| `x` | Switch to SQL view |
| `a` | Add transformation |
| `q` | Quit tile |
| `n` | New pipeline |
| `Enter` | Execute pipeline |
| `Delete` | Remove selected step |
| `Up/Down` | Navigate steps |

## Common Issues

### Tile Not Showing in List

Make sure the binary is in your PATH:
```bash
export PATH=$PATH:$(pwd)/target/release
```

### Data Not Loading

- Check file format (CSV, Parquet, JSON, TSV supported)
- Verify file exists and is readable
- Check for encoding issues

### Keybindings Not Working

- Restart the tile after configuration changes
- Check for conflicts with Zellij keybindings

## Next Steps

- Read the full [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md) for detailed usage
- Check [layouts/example.zellij/layouts/data_workspace.kdl](layouts/example.zellij/layouts/data_workspace.kdl) for layout examples
- Review [nustage_tile.toml](nustage_tile.toml) to customize keybindings and colors

## Resources

- [Nustage Main Project](../README.md)
- [Zellij Documentation](https://zellij.dev/)
- [Power Query M Language](https://learn.microsoft.com/en-us/power-query-m/)
- [DuckDB Documentation](https://duckdb.org/docs/)