//! Nustage Zellij Tile
//!
//! A Zellij tile that provides data exploration and transformation capabilities
//! using Nustage's transformation engine.

use nustage::data::{ColumnSchema, load_dataframe};
use nustage::transformations::{StepType, TransformationPipeline, TransformationStep};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use zellij_tile_utils::{Event, EventWrapper, Key, MouseEvent, Pane, Tile, TileConfig};

/// Main Nustage tile state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NustageState {
    /// Current pipeline
    pub pipeline: TransformationPipeline,
    /// Currently loaded dataframe
    pub data: Option<zellij_tile_utils::DataFrame>,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Selected step index
    pub selected_step: usize,
    /// Search query
    pub search_query: String,
}

/// View modes for the tile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewMode {
    /// Show all steps
    Steps,
    /// Show data preview
    Data,
    /// Show schema
    Schema,
    /// Show SQL transparency
    SQL,
}

/// Nustage tile implementation
pub struct NustageTile {
    state: NustageState,
    config: TileConfig,
}

impl NustageTile {
    /// Create a new Nustage tile
    pub fn new() -> Self {
        let pipeline = TransformationPipeline::new("Nustage Pipeline".to_string());
        let state = NustageState {
            pipeline,
            data: None,
            view_mode: ViewMode::Steps,
            selected_step: 0,
            search_query: String::new(),
        };

        let config = TileConfig::default();

        NustageTile { state, config }
    }

    /// Load data from a file
    pub fn load_data(&mut self, path: PathBuf) -> Result<(), String> {
        match load_dataframe(&path) {
            Ok(df) => {
                self.state.data = Some(df);
                self.state.pipeline.input_schema = df.schema().clone();
                Ok(())
            }
            Err(e) => Err(format!("Failed to load data: {}", e)),
        }
    }

    /// Add a transformation step
    pub fn add_step(&mut self, step_type: StepType) -> Result<(), String> {
        let step = match step_type {
            StepType::SelectColumns(columns) => TransformationStep {
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Select {}", columns.len()),
                step_type: StepType::SelectColumns(columns),
                parameters: HashMap::new(),
                output_schema: vec![],
            },
            StepType::FilterRows(column, condition) => TransformationStep {
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Filter {}", column),
                step_type: StepType::FilterRows(column, condition),
                parameters: HashMap::new(),
                output_schema: vec![],
            },
            StepType::GroupBy(columns, aggregations) => TransformationStep {
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("Group {}", columns.join(",")),
                step_type: StepType::GroupBy(columns, aggregations),
                parameters: HashMap::new(),
                output_schema: vec![],
            },
            _ => return Err("Step type not yet implemented".to_string()),
        };

        self.state.pipeline.add_step(step);
        Ok(())
    }

    /// Remove a step
    pub fn remove_step(&mut self, index: usize) {
        if index < self.state.pipeline.steps.len() {
            self.state.pipeline.remove_step(index);
        }
    }

    /// Get the current pipeline
    pub fn pipeline(&self) -> &TransformationPipeline {
        &self.state.pipeline
    }

    /// Get the current data
    pub fn data(&self) -> Option<&zellij_tile_utils::DataFrame> {
        self.state.data.as_ref()
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.state.view_mode = mode;
    }

    /// Set selected step
    pub fn set_selected_step(&mut self, index: usize) {
        self.state.selected_step = index.min(self.state.pipeline.steps.len());
    }

    /// Update search query
    pub fn update_search(&mut self, query: String) {
        self.state.search_query = query;
    }
}

impl Tile for NustageTile {
    fn new() -> Self {
        Self::new()
    }

    fn process_event(&mut self, event: EventWrapper) {
        match event {
            EventWrapper::Event(Event::Key(key)) => {
                self.handle_key_event(key);
            }
            EventWrapper::Event(Event::Mouse(mouse)) => {
                self.handle_mouse_event(mouse);
            }
            EventWrapper::Event(Event::Resize { width, height }) => {
                self.config.width = width;
                self.config.height = height;
            }
            EventWrapper::Event(Event::Tick) => {
                // Handle tick events if needed
            }
        }
    }

    fn render(&self, pane: &Pane) {
        self.render_steps(pane);
        self.render_data(pane);
        self.render_schema(pane);
        self.render_sql(pane);
    }

    fn config(&self) -> TileConfig {
        self.config.clone()
    }
}

impl NustageTile {
    /// Handle keyboard events
    fn handle_key_event(&mut self, key: Key) {
        match key {
            Key::Char('q') => {
                // Exit tile
            }
            Key::Char('s') => {
                self.state.view_mode = ViewMode::Steps;
            }
            Key::Char('d') => {
                self.state.view_mode = ViewMode::Data;
            }
            Key::Char('z') => {
                self.state.view_mode = ViewMode::Schema;
            }
            Key::Char('x') => {
                self.state.view_mode = ViewMode::SQL;
            }
            Key::Up => {
                self.state.selected_step = self.state.selected_step.saturating_sub(1);
            }
            Key::Down => {
                self.state.selected_step = self
                    .state
                    .selected_step
                    .min(self.state.pipeline.steps.len());
            }
            Key::Delete => {
                self.remove_step(self.state.selected_step);
            }
            Key::Enter => {
                // Execute pipeline
            }
            _ => {}
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        // Handle mouse clicks for navigation
    }

    /// Render the steps view
    fn render_steps(&self, pane: &Pane) {
        let title = "Nustage Steps";
        let steps = &self.state.pipeline.steps;

        pane.set_title(title);

        // Header
        pane.set_style("bold cyan");
        pane.print(format!("{}: {}", title, steps.len()));

        // Step list
        pane.set_style("reset");
        pane.print("\n");

        for (i, step) in steps.iter().enumerate() {
            if i == self.state.selected_step {
                pane.set_style("bold yellow");
            } else {
                pane.set_style("reset");
            }

            pane.print(format!("  [{}] {}", i + 1, step.name));

            if i == self.state.selected_step {
                pane.set_style("reset");
            }
        }

        // Instructions
        pane.print("\n");
        pane.set_style("dim gray");
        pane.print("  s - Steps | d - Data | z - Schema | x - SQL | q - Exit");
        pane.set_style("reset");
    }

    /// Render the data view
    fn render_data(&self, pane: &Pane) {
        if self.state.view_mode != ViewMode::Data {
            return;
        }

        let data = match self.state.data.as_ref() {
            Some(df) => df,
            None => {
                pane.set_title("No Data");
                pane.set_style("bold red");
                pane.print("No data loaded. Press 'l' to load data.");
                pane.set_style("reset");
                return;
            }
        };

        pane.set_title("Data Preview");

        // Show first few rows
        pane.set_style("bold cyan");
        pane.print("Data Preview");

        pane.set_style("reset");
        pane.print("\n");

        // Simple header
        let columns = data.schema().names();
        pane.print(format!("  {}", columns.join(" | ")));

        // Show first 5 rows
        for row in data.iter().take(5) {
            pane.print("\n");
            pane.set_style("dim gray");
            pane.print("  |");

            for val in row {
                pane.set_style("reset");
                pane.print(format!(" {}", val));
            }
        }

        pane.print("\n");
        pane.set_style("dim gray");
        pane.print(format!("  (Showing first 5 rows of {} total)", data.len()));
    }

    /// Render the schema view
    fn render_schema(&self, pane: &Pane) {
        if self.state.view_mode != ViewMode::Schema {
            return;
        }

        let schema = match self.state.data.as_ref() {
            Some(df) => df.schema(),
            None => {
                pane.set_title("No Schema");
                pane.set_style("bold red");
                pane.print("No data loaded.");
                pane.set_style("reset");
                return;
            }
        };

        pane.set_title("Schema");

        pane.set_style("bold cyan");
        pane.print("Schema");

        pane.set_style("reset");
        pane.print("\n");

        for (i, col) in schema.iter().enumerate() {
            pane.print(format!(
                "  {:>3}. {} ({})",
                i + 1,
                col.name(),
                col.data_type()
            ));
        }
    }

    /// Render the SQL transparency view
    fn render_sql(&self, pane: &Pane) {
        if self.state.view_mode != ViewMode::SQL {
            return;
        }

        pane.set_title("SQL Transparency");

        pane.set_style("bold cyan");
        pane.print("Generated SQL");

        pane.set_style("reset");
        pane.print("\n");

        // Generate SQL from pipeline
        let sql = self.state.pipeline.to_sql();
        pane.print(format!("  {}", sql));
    }
}
