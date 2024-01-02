// Helper functions and datastructures are here :3
use std::fs::File;
use anyhow::Result;

// Trust implementer to design this well
pub trait CompressionFn{
    /// encode the data at input and write to output
    fn encode(&mut self, input:&mut File, ext:&str, output:&mut File) -> Result<()>;
    /// decode the file at input and write to output
    fn decode(&mut self, input:&mut File, output:&mut File) -> Result<()>;
}
