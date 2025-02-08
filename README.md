# Wavglitch

Program that divides audio into segments and processes them in order to create glitch-like effects.

## Options

- **-o, --output <path>**: Output WAV file path (defaults to 'out.wav')
- **-t, --tempo <value>**: Tempo, 1.0 to 4095.0 (defaults to 100.0)
- **-l, --length <value>**: Length of a single segment, relative note value in x/y format (defaults to 1/16)
- **-s, --silence <prob>**: Probability of silencing segment, 0.0 to 1.0 (defaults to 0.0)
- **-w, --swap <prob>**: Probability of swapping segment, 0.0 to 1.0 (defaults to 0.0)
- **-r, --reverse <prob>**: Probability of reversing segment, 0.0 to 1.0 (defaults to 0.0)
- **-p, --repeat <prob>**: Probability of repeating segment, 0.0 to 1.0 (defaults to 0.0)
- **-a, --range <max>**: Maximal swap range, 1 to 65535 (defaults to 8)
- **-n, --number <max>**: Maximal number of repetitions, 1 to 65535 (defaults to 8)
- **-c, --channels**: Process each channel separately (defaults to false)
- **-h, --help**: Print help
- **-V, --version**: Print version

## Examples

Process 'in.wav', dividing it into segments with a length of a 1/32 note in 120 BPM and output result to 'processed.wav' with 10% chance of repeating a segment up to 20 times. Use defaults for other options.

```sh
wavglitch in.wav -o processed.wav -t 120 -l 1/32 -p 0.1 -n 20
```

Process 'in.wav', dividing it into segments with a length of a 1/4 note in 200 BPM with 5% chance of swapping segments in default range of 8 and 15% chance of reversing a segment. Process each channel separately and use defaults (including default output path) for other options.

```sh
wavglitch in.wav -o processed.wav -t 200 -l 1/4 -w 0.05 -r 0.15 -c
```

## Building

Install [Rust](https://www.rust-lang.org/tools/install).

```sh
$ git clone https://github.com/Acithikeih/wavglitch
$ cd wavglitch
$ cargo build --release
```

Copy `target/release/wavglitch` executable to a directory in the `PATH` variable.
