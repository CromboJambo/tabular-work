# Nustage Zellij Integration Guide

## Overview

This guide explains how to integrate the Nustage Zellij tile into your development workflow. The tile provides a terminal-native interface for data exploration and transformation using Nustage's transformation engine.

## Prerequisites

### Required Software

1. **Rust 1.70+**
   ```bash
   cargo --version
   ```

2. **Zellij 0.40+**
   ```bash
   zellij --version
   ```

3. **Nustage Core Library**
   - The tile depends on the main Nustage library
   - Ensure it's available in your path

### Optional Dependencies

- **DuckDB** (for SQL generation and query folding)
- **Polars** (for data loading)
- **zellij-tile-utils** (for tile interface)

## Installation

### Building the Tile

1. Navigate to the tile directory:
   ```bash
   cd nustage/zellij-tile
   ```

2. Build the release version:
   ```bash
   cargo build --release
   ```

3. The binary will be at:
   ```
   target/release/nustage-zellij-tile
   ```

### Installing Zellij

If you don't have Zellij installed:

```bash
cargo install zellij
```

## Quick Start

### Step 1: Start a Zellij Session

```bash
zellij -S mysession
```

### Step 2: Add the Nustage Tile

1. Press `Ctrl+Shift+I` to open the tile list
2. Type `nustage` and select it
3. Press `Enter` to add it to your session

### Step 3: Load Data

1. In the Nustage tile, press `l`
2. Navigate to your data file (CSV, Parquet, JSON, or TSV)
3. Press `Enter`

### Step 4: Add Transformations

1. Navigate steps using `Up` and `Down` arrows
2. Press `a` to add a new transformation
3. Select the transformation type

### Step 5: View Results

- `s` - View steps
- `d` - View data
- `z` - View schema
- `x` - View SQL

## Configuration

### Customizing Keybindings

Edit the `nustage_tile.toml` file to customize keybindings:

```toml
[keybindings]
file_load = "l"  # Change this to your preferred key
filter_rows = "f"
# ... more keybindings
```

### Customizing Colors

Modify the color scheme in the same file:

```toml
[colors]
primary = "cyan"  # Change to your preferred color
secondary = "green"
# ... more colors
```

### Customizing Status Bar

Adjust status bar settings:

```toml
[status_bar]
enabled = true
position = "bottom"
left = "file_info"
center = "pipeline_status"
right = "view_mode"
spacing = 2
```

## Layouts

### Default Layout

The tile works well in a simple vertical split:

```kdl
layout {
    split {
        direction: "Vertical"
        
        pane {
            command: "nustage-zellij-tile"
            focus: true
        }
        
        pane {
            command: "zsh"
        }
    }
}
```

### Data Workspace Layout

A recommended setup for data analysis:

```kdl
layout {
    split {
        direction: "Horizontal"
        
        pane {
            command: "nustage-zellij-tile"
            focus: true
        }
        
        pane {
            split {
                direction: "Vertical"
                
                pane {
                    command: "nvim"
                }
                
                pane {
                    command: "zsh"
                }
            }
        }
    }
}
```

## Common Workflows

### Workflow 1: CSV Analysis

1. Load CSV file: `l`
2. View schema: `z`
3. Add filter: `a` → Filter Rows
4. View data: `d`
5. Add aggregation: `a` → Group By
6. View results: `d`

### Workflow 2: M Code Export

1. Load data: `l`
2. Build transformations: `a`
3. View SQL: `x`
4. Copy the SQL query
5. Use in external tools

### Workflow 3: Pipeline Management

1. Start new pipeline: `n`
2. Add transformations: `a`
3. Navigate between steps: `Up`/`Down`
4. Remove steps: `Delete`
5. Execute: `Enter`

## Troubleshooting

### Tile Not Appearing in Tile List

1. Ensure the binary is in your PATH
2. Check that Zellij can find the tile
3. Verify the binary name: `nustage-zellij-tile`

### Data Not Loading

1. Check file format (CSV, Parquet, JSON, TSV)
2. Verify file exists and is readable
3. Check for encoding issues
4. Review error messages in the tile

### Keybindings Not Working

1. Check configuration file is valid TOML
2. Verify keybinding conflicts with Zellij
3. Restart the tile after configuration changes

### Performance Issues

1. Large datasets may be slow
2. Consider using Parquet format for better performance
3. Reduce the number of transformations
4. Use SQL view to optimize queries

## Advanced Features

### Query Folding

The tile supports query folding for deferred execution:

1. Build transformation pipeline
2. View SQL output: `x`
3. The SQL shows how DuckDB will optimize the query

### M Code Generation

Generate Power Query M code for Excel:

1. Build transformations
2. View SQL output: `x`
3. Copy M code for use in Power Query

### Session Persistence

Zellij sessions persist automatically:

1. Start a named session: `zellij -S mysession`
2. Add tiles and configure layouts
3. Close Zellij
4. Resume later: `zellij -S mysession`

## Development

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Debug Mode

```bash
RUST_LOG=debug cargo run
```

### Adding New Transformations

1. Modify `src/lib.rs`
2. Implement transformation logic
3. Add keybindings in configuration
4. Update documentation

## Contributing

### Code Style

- Follow Rust conventions
- Use `cargo fmt` to format code
- Use `cargo clippy` to check for issues

### Testing

- Add unit tests for new features
- Test with various data formats
- Verify keybindings work correctly

## Resources

### Documentation

- [Nustage Main Project](../README.md)
- [Zellij Documentation](https://zellij.dev/)
- [Power Query M Language](https://learn.microsoft.com/en-us/power-query-m/)
- [DuckDB Documentation](https://duckdb.org/docs/)

### Examples

- [Example Layouts](layouts/)
- [Example Configurations](nustage_tile.toml)

## Support

### Getting Help

1. Check the main Nustage documentation
2. Review Zellij documentation
3. Check for known issues in the repository
4. Open an issue on GitHub

### Reporting Issues

When reporting issues, include:

- Zellij version
- Nustage tile version
- Operating system
- Error messages
- Steps to reproduce

## License

MIT License - See LICENSE file in parent directory

## Changelog

### Version 0.1.0

- Initial release
- Basic data loading
- Transformation pipeline
- Multiple view modes
- M code generation
- SQL transparency
</arg_value>