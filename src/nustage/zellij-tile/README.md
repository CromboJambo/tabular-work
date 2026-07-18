# Nustage Zellij Tile

A Zellij tile that provides data exploration and transformation capabilities using Nustage's transformation engine.

## Overview

The Nustage Zellij Tile integrates Nustage's step-based data transformation system with Zellij's terminal workspace. It allows you to:

- Load and explore data files (CSV, Parquet, JSON, TSV)
- Build transformation pipelines with named, reversible steps
- Preview data and schemas in real-time
- Generate Power Query M code
- View generated SQL queries
- Navigate and modify transformation pipelines

## Features

### Data Loading

- Load CSV files
- Load Parquet files
- Load JSON files
- Load TSV files
- Automatic schema inference

### Transformation Pipeline

- Add transformation steps
- Remove transformation steps
- Navigate steps with arrow keys
- View step details
- Execute pipeline

### Views

- **Steps View** (`s`) - Show all transformation steps
- **Data View** (`d`) - Preview the current dataset
- **Schema View** (`z`) - Show column schemas and types
- **SQL View** (`x`) - Display generated SQL queries
- **Help View** (`h`) - Show keybindings and help

### Code Generation

- Generate Power Query M code from transformations
- Export M code for use in Excel
- Generate SQL queries for DuckDB backend

## Installation

### Prerequisites

- Rust 1.70+
- Zellij 0.40+

### Building

```bash
cd zellij-tile
cargo build --release
```

### Installing Zellij

```bash
cargo install zellij
```

## Usage

### Running the Tile

1. Start Zellij:
   ```bash
   zellij -S mysession
   ```

2. Add the tile: `Ctrl+Shift+I`

3. Select the tile: `nustage-zellij-tile`

### Basic Workflow

1. **Load Data** (`l`)
   - Press `l` to load a data file
   - Navigate to the file and press Enter

2. **Add Transformations**
   - Use arrow keys to navigate steps
   - Press `a` to add a new transformation
   - Select the transformation type

3. **View Results**
   - Press `d` to view data
   - Press `z` to view schema
   - Press `s` to view steps
   - Press `x` to view SQL

4. **Export**
   - View M code in the SQL view
   - Copy and use in Excel

## Keybindings

| Key | Action |
|-----|--------|
| `l` | Load data file |
| `s` | Switch to Steps view |
| `d` | Switch to Data view |
| `z` | Switch to Schema view |
| `x` | Switch to SQL view |
| `h` | Show help |
| `q` | Quit tile |
| `n` | New pipeline |
| `Enter` | Execute pipeline |
| `Delete` | Remove selected step |
| `Up/Down` | Navigate steps |
| `Left/Right` | Navigate in views |

## Configuration

The tile uses a TOML configuration file (`nustage_tile.toml`) to customize:

- Keybindings
- Color scheme
- Status bar settings
- Window options
- Terminal settings

## Architecture

### Components

- **Nustage Tile** - Main tile implementation
- **Nustage Utils** - Utility functions for Zellij integration
- **Nustage Core** - Transformation pipeline engine

### Integration Points

- Uses `zellij-tile-utils` for tile interface
- Integrates with `nustage` core library
- Leverages `DuckDB` for SQL generation
- Uses `Polars` for data loading

## Development

### Project Structure

```
zellij-tile/
├── Cargo.toml
├── README.md
├── nustage_tile.toml
└── src/
    ├── lib.rs          - Main tile implementation
    └── main.rs         - Entry point
```

### Running Tests

```bash
cargo test
```

### Debug Build

```bash
cargo build
```

## Roadmap

- [ ] Implement query folding for deferred execution
- [ ] Add M code interpreter
- [ ] Support for Excel files (.xlsx)
- [ ] Add visualization views
- [ ] Session management and persistence
- [ ] WASM plugin support
- [ ] Custom transformation plugins

## Contributing

Contributions are welcome! Please see the main Nustage project for contribution guidelines.

## License

MIT License - See LICENSE file in parent directory

## Resources

- [Nustage Main Project](../README.md)
- [Zellij Documentation](https://zellij.dev/)
- [Power Query M Language](https://learn.microsoft.com/en-us/power-query-m/)
- [DuckDB Documentation](https://duckdb.org/docs/)