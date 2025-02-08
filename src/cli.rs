pub use clap::Parser;
use std::path::PathBuf;
use yansi::Paint;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Program that divides audio into segments and processes them in order to create glitch-like effects.\n\
        You can find program's source code as well as report issue at https://github.com/Acithikeih/wavglitch.",
    after_help = format!("{}\n  Process `in.wav`, dividing it into segments with a length of a 1/32 note in 120 BPM and output\n  \
        result to `processed.wav` with 10% chance of repeating a segment up to 20 times.\n  \
        Use defaults for other options.\n  \
        $ wavglitch in.wav -o processed.wav -t 120 -l 1/32 -p 0.1 -n 20", "Example:".bold().underline()),
    arg_required_else_help = true,
    args_override_self = true
)]
pub struct Cli {
    /// Input WAV file path
    #[arg(value_name = "input")]
    input: PathBuf,
    /// Output WAV file path (defaults to `out.wav`)
    #[arg(short = 'o', long = "output", value_name = "path")]
    output: Option<PathBuf>,
    /// Tempo, 1.0 to 4095.0 (defaults to 100.0)
    #[arg(short = 't', long = "tempo", value_name = "value", value_parser = Cli::tempo_parser)]
    tempo: Option<f64>,
    /// Length of a single segment, relative note value in x/y format (defaults to 1/16)
    #[arg(short = 'l', long = "length", value_name = "value", value_parser = Cli::segment_parser)]
    segment_length: Option<f64>,
    /// Probability of silencing segment, 0.0 to 1.0 (defaults to 0.0)
    #[arg(short = 's', long = "silence", value_name = "prob", value_parser = Cli::probability_parser)]
    prob_silence: Option<f64>,
    /// Probability of swapping segment, 0.0 to 1.0 (defaults to 0.0)
    #[arg(short = 'w', long = "swap", value_name = "prob", value_parser = Cli::probability_parser)]
    prob_swap: Option<f64>,
    /// Probability of reversing segment, 0.0 to 1.0 (defaults to 0.0)
    #[arg(short = 'r', long = "reverse", value_name = "prob", value_parser = Cli::probability_parser)]
    prob_reverse: Option<f64>,
    /// Probability of repeating segment, 0.0 to 1.0 (defaults to 0.0)
    #[arg(short = 'p', long = "repeat", value_name = "prob", value_parser = Cli::probability_parser)]
    prob_repeat: Option<f64>,
    /// Maximal swap range, 1 to 65535 (defaults to 8)
    #[arg(short = 'a', long = "range", value_name = "max", value_parser = clap::value_parser!(u16).range(1..))]
    max_swap: Option<u16>,
    /// Maximal number of repetitions, 1 to 65535 (defaults to 8)
    #[arg(short = 'n', long = "number", value_name = "max", value_parser = clap::value_parser!(u16).range(1..))]
    max_repeat: Option<u16>,
    /// Process each channel separately (defaults to false)
    #[arg(short = 'c', long = "channels")]
    each_channel_separately: bool,
}

#[derive(Copy, Clone)]
pub struct CliConfig {
    pub tempo: f64,
    pub segment_length: f64,
    pub prob_silence: f64,
    pub prob_swap: f64,
    pub prob_reverse: f64,
    pub prob_repeat: f64,
    pub max_swap: u16,
    pub max_repeat: u16,
    pub each_channel_separately: bool,
}

impl Cli {
    pub fn input(&self) -> PathBuf {
        self.input.clone()
    }

    pub fn output(&self) -> PathBuf {
        self.output.clone().unwrap_or("out.wav".into())
    }

    pub fn config(&self) -> CliConfig {
        CliConfig {
            tempo: self.tempo.unwrap_or(100.),
            segment_length: self.segment_length.unwrap_or(0.0625),
            prob_silence: self.prob_silence.unwrap_or(0.),
            prob_swap: self.prob_swap.unwrap_or(0.),
            prob_reverse: self.prob_reverse.unwrap_or(0.),
            prob_repeat: self.prob_repeat.unwrap_or(0.),
            max_swap: self.max_swap.unwrap_or(8),
            max_repeat: self.max_repeat.unwrap_or(8),
            each_channel_separately: self.each_channel_separately,
        }
    }

    pub fn defaults(&self) -> String {
        let mut string = String::new();
        if self.output.is_none() {
            string.push_str("Using default value (`out.wav`) for output path\n");
        }
        if self.tempo.is_none() {
            string.push_str("Using default value (100) for tempo\n");
        }
        if self.segment_length.is_none() {
            string.push_str("Using default value (1/16) for segment length\n");
        }
        if self.prob_silence.is_none() {
            string.push_str("Using default value (0.0) for probability of silencing\n");
        }
        if self.prob_swap.is_none() {
            string.push_str("Using default value (0.0) for probability of swapping\n");
        }
        if self.prob_reverse.is_none() {
            string.push_str("Using default value (0.0) for probability of reversing\n");
        }
        if self.prob_repeat.is_none() {
            string.push_str("Using default value (0.0) for probability of repeating\n");
        }
        if self.max_swap.is_none() {
            string.push_str("Using default value (8) for maximal swap range\n");
        }
        if self.max_repeat.is_none() {
            string.push_str("Using default value (8) for maximal number of repetitions\n");
        }
        string.pop();
        string
    }

    fn tempo_parser(s: &str) -> Result<f64, String> {
        let tempo: f64 = s.parse().map_err(|e| format!("{e}"))?;

        if (1f64..=4095f64).contains(&tempo) {
            Ok(tempo)
        } else {
            Err(format!("{tempo} is not in 1.0..=4095.0"))
        }
    }

    fn segment_parser(s: &str) -> Result<f64, String> {
        let v: Vec<_> = s.split('/').collect();
        if v.len() != 2 {
            return Err("segment length must be in x/y format".to_string());
        }
        let n: u16 = v[0].parse().map_err(|e| format!("{e}"))?;
        let d: u16 = v[1].parse().map_err(|e| format!("{e}"))?;
        if n == 0 || d == 0 {
            return Err("both numbers must be in 1..=65535".to_string());
        }
        Ok(n as f64 / d as f64)
    }

    fn probability_parser(s: &str) -> Result<f64, String> {
        let probability: f64 = s.parse().map_err(|e| format!("{e}"))?;

        if (0f64..=1f64).contains(&probability) {
            Ok(probability)
        } else {
            Err(format!("{probability} is not in 0.0..=1.0"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_all() {
        let cli = Cli::try_parse_from(["test", "in.wav"]).unwrap();

        assert_eq!(
            cli.defaults(),
            "Using default value (`out.wav`) for output path\n\
             Using default value (100) for tempo\n\
             Using default value (1/16) for segment length\n\
             Using default value (0.0) for probability of silencing\n\
             Using default value (0.0) for probability of swapping\n\
             Using default value (0.0) for probability of reversing\n\
             Using default value (0.0) for probability of repeating\n\
             Using default value (8) for maximal swap range\n\
             Using default value (8) for maximal number of repetitions"
                .to_string()
        );
    }

    #[test]
    fn defaults_none() {
        let cli = Cli::try_parse_from([
            "test", "in.wav", "-o", "out.wav", "-t", "1", "-l", "1/1", "-s", "1", "-w", "1", "-r",
            "1", "-p", "1", "-a", "1", "-n", "1",
        ])
        .unwrap();

        assert_eq!(cli.defaults(), "".to_string());
    }

    #[test]
    fn tempo_parser_not_float() {
        assert_eq!(
            Cli::tempo_parser(&"float"),
            Err("invalid float literal".to_string())
        );
    }

    #[test]
    fn tempo_parser_lesser() {
        assert_eq!(
            Cli::tempo_parser(&"0.25"),
            Err("0.25 is not in 1.0..=4095.0".to_string())
        );
    }

    #[test]
    fn tempo_parser_greater() {
        assert_eq!(
            Cli::tempo_parser(&"5000"),
            Err("5000 is not in 1.0..=4095.0".to_string())
        );
    }

    #[test]
    fn tempo_parser_nan() {
        assert_eq!(
            Cli::tempo_parser(&"NaN"),
            Err("NaN is not in 1.0..=4095.0".to_string())
        );
    }

    #[test]
    fn tempo_parser_inf() {
        assert_eq!(
            Cli::tempo_parser(&"inf"),
            Err("inf is not in 1.0..=4095.0".to_string())
        );
    }

    #[test]
    fn tempo_parser_ok() {
        assert_eq!(Cli::tempo_parser(&"100"), Ok(100f64));
    }

    #[test]
    fn segment_parser_two_div() {
        assert_eq!(
            Cli::segment_parser(&"1/2/4"),
            Err("segment length must be in x/y format".to_string())
        );
    }

    #[test]
    fn segment_parser_no_div() {
        assert_eq!(
            Cli::segment_parser(&"."),
            Err("segment length must be in x/y format".to_string())
        );
    }

    #[test]
    fn segment_parser_not_int() {
        assert_eq!(
            Cli::segment_parser(&"a/4"),
            Err("invalid digit found in string".to_string())
        );
    }

    #[test]
    fn segment_parser_zero_n() {
        assert_eq!(
            Cli::segment_parser(&"0/4"),
            Err("both numbers must be in 1..=65535".to_string())
        );
    }

    #[test]
    fn segment_parser_zero_d() {
        assert_eq!(
            Cli::segment_parser(&"1/0"),
            Err("both numbers must be in 1..=65535".to_string())
        );
    }

    #[test]
    fn segment_parser_ok() {
        assert_eq!(Cli::segment_parser(&"1/4"), Ok(0.25f64));
    }

    #[test]
    fn probability_parser_not_float() {
        assert_eq!(
            Cli::probability_parser(&"float"),
            Err("invalid float literal".to_string())
        );
    }

    #[test]
    fn probability_parser_lesser() {
        assert_eq!(
            Cli::probability_parser(&"-0.25"),
            Err("-0.25 is not in 0.0..=1.0".to_string())
        );
    }

    #[test]
    fn probability_parser_greater() {
        assert_eq!(
            Cli::probability_parser(&"1.25"),
            Err("1.25 is not in 0.0..=1.0".to_string())
        );
    }

    #[test]
    fn probability_parser_nan() {
        assert_eq!(
            Cli::probability_parser(&"NaN"),
            Err("NaN is not in 0.0..=1.0".to_string())
        );
    }

    #[test]
    fn probability_parser_inf() {
        assert_eq!(
            Cli::probability_parser(&"inf"),
            Err("inf is not in 0.0..=1.0".to_string())
        );
    }

    #[test]
    fn probability_parser_ok() {
        assert_eq!(Cli::probability_parser(&"0.5"), Ok(0.5f64));
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
