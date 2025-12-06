use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use serde::{Deserialize, Serialize};

use statn::models::cd_ma::CoordinateDescent;
use crate::indicators::IndicatorSpec;
use crate::config::Config;

/// Container for a saved model
#[derive(Serialize, Deserialize)]
pub struct SavedModel {
    /// Trained model weights and stats
    pub model: CoordinateDescent,
    /// Specifications for indicators used in the model
    pub specs: Vec<IndicatorSpec>,
    /// Configuration used to generate the model (optional metadata)
    pub config: Config,
}

impl SavedModel {
    /// Create a new SavedModel container
    pub fn new(model: CoordinateDescent, specs: Vec<IndicatorSpec>, config: Config) -> Self {
        Self {
            model,
            specs,
            config,
        }
    }

    /// Save model to a JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path)
            .with_context(|| format!("Failed to create model file: {}", path.display()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)
            .with_context(|| "Failed to serialize model to JSON")?;
        println!("Model saved to {}", path.display());
        Ok(())
    }

    /// Load model from a JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!("Failed to open model file: {}", path.display()))?;
        let reader = BufReader::new(file);
        let model = serde_json::from_reader(reader)
            .with_context(|| "Failed to deserialize model from JSON")?;
        Ok(model)
    }
}
