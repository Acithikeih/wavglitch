use crate::cli::Cli;
use crate::segment_layout::SegmentLayout;
use crate::wav_reader::WavReader;
use crate::wav_writer::WavWriter;
use anyhow::{anyhow, Result};
use std::io::{self, Write};
use yansi::Condition;

pub fn run(cli: Cli) -> Result<()> {
    if cli.input() == cli.output() {
        return Err(anyhow!("input path is the same as output path"));
    }

    let mut reader = WavReader::open(cli.input())?;
    let mut writer = WavWriter::create(cli.output(), reader.spec())?;

    println!("{}", cli.defaults());

    let layout = SegmentLayout::build(cli.config(), reader.config());

    let mut threshold = 0;

    for mut slice in layout {
        match reader.spec().sample_format {
            hound::SampleFormat::Int => writer.write(&reader.read::<i32>(&mut slice)?),
            hound::SampleFormat::Float => writer.write(&reader.read::<f32>(&mut slice)?),
        }?;
        if slice.percentage() as u8 > threshold {
            if Condition::stdout_is_tty() {
                print!("\rProcessing... {:.2}%", slice.percentage());
                io::stdout().flush()?;
            }
            threshold = slice.percentage() as u8;
        }
    }
    println!("\nDone");

    writer.finalize()?;

    Ok(())
}
