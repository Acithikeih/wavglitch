use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::path::Path;

pub struct WavWriter {
    writer: hound::WavWriter<BufWriter<File>>,
}

impl WavWriter {
    pub fn create<P: AsRef<Path>>(path: P, spec: hound::WavSpec) -> Result<WavWriter> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .context("when creating output file")?;
        let writer = hound::WavWriter::new(BufWriter::new(file), spec)
            .context("when creating output file")?;
        Ok(WavWriter { writer })
    }

    pub fn write<S: hound::Sample + Copy>(&mut self, samples: &[S]) -> Result<()> {
        for sample in samples {
            self.writer
                .write_sample(*sample)
                .context("when writing to output file")?;
        }
        Ok(())
    }

    pub fn finalize(self) -> Result<()> {
        self.writer
            .finalize()
            .context("when finalizing output file")
    }
}
