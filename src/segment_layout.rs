use crate::cli::CliConfig;
use crate::wav_reader::WavConfig;
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct SegmentLayout {
    segments: Vec<Vec<Segment>>,
    index: usize,
    segment_len: usize,
}

impl SegmentLayout {
    pub fn build(cli_config: CliConfig, wav_config: WavConfig) -> SegmentLayout {
        let mut segments = vec![vec![]; wav_config.channels as usize];

        if cli_config.each_channel_separately {
            for channel in &mut segments {
                *channel = Self::build_channel(cli_config, wav_config);
            }
        } else {
            let channel = Self::build_channel(cli_config, wav_config);
            segments.fill(channel);
        }

        let segment_len = Self::segment_len(
            wav_config.sample_rate,
            cli_config.tempo,
            cli_config.segment_length,
        ) as usize;

        SegmentLayout {
            segments,
            index: 0,
            segment_len,
        }
    }

    fn build_channel(cli_config: CliConfig, wav_config: WavConfig) -> Vec<Segment> {
        let segment_len = Self::segment_len(
            wav_config.sample_rate,
            cli_config.tempo,
            cli_config.segment_length,
        );
        let (segment_count, is_incomplete) = Self::segment_count(wav_config.duration, segment_len);
        let mut channel = Vec::with_capacity(segment_count);
        let mut rng = thread_rng();

        for i in 0..segment_count {
            channel.push(Segment::new(
                i as u32 * segment_len,
                rng.gen_bool(cli_config.prob_reverse),
                rng.gen_bool(cli_config.prob_silence),
            ));
        }

        for i in 0..segment_count {
            if rng.gen_bool(cli_config.prob_swap) {
                let swap = (i + rng.gen_range(1..=cli_config.max_swap as usize)).min(
                    segment_count
                        - if is_incomplete && i != segment_count - 1 {
                            2
                        } else {
                            1
                        },
                );
                channel.swap(i, swap);
            }
        }

        let mut i = 0;

        while i < segment_count {
            if rng.gen_bool(cli_config.prob_repeat) {
                let repeat = (i + rng.gen_range(1..=cli_config.max_repeat as usize)).min(
                    segment_count
                        - if is_incomplete && i != segment_count - 1 {
                            2
                        } else {
                            1
                        },
                );
                let tmp = channel[i];
                channel[i..=repeat].fill(tmp);
                i = repeat + 1;
            } else {
                i += 1;
            }
        }

        channel
    }

    fn segment_len(sample_rate: u32, tempo: f64, note_value: f64) -> u32 {
        (sample_rate as f64 * 240. * note_value / tempo) as u32
    }

    fn segment_count(duration: u32, segment_len: u32) -> (usize, bool) {
        (
            (duration as f64 / segment_len as f64).ceil() as usize,
            duration % segment_len != 0,
        )
    }
}

impl Iterator for SegmentLayout {
    type Item = SegmentSlice;

    fn next(&mut self) -> Option<Self::Item> {
        let mut vec = Vec::with_capacity(self.segments.len());

        for ch in &self.segments {
            vec.push(ch.get(self.index).cloned()?);
        }

        self.index += 1;

        Some(SegmentSlice::new(
            vec,
            self.segment_len,
            100. * self.index as f64 / self.segments[0].len() as f64,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct SegmentSlice {
    segments: Vec<Segment>,
    index: usize,
    segment_len: usize,
    percentage: f64,
}

impl SegmentSlice {
    pub fn new(segments: Vec<Segment>, segment_len: usize, percentage: f64) -> SegmentSlice {
        SegmentSlice {
            segments,
            index: 0,
            segment_len,
            percentage,
        }
    }

    pub fn segment_len(&self) -> usize {
        self.segment_len
    }

    pub fn percentage(&self) -> f64 {
        self.percentage
    }
}

impl Iterator for SegmentSlice {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        let segment = self.segments.get(self.index).cloned();
        self.index += 1;
        segment
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Segment {
    offset: u32,
    reverse: bool,
    silence: bool,
}

impl Segment {
    pub fn new(offset: u32, reverse: bool, silence: bool) -> Segment {
        Segment {
            offset,
            reverse,
            silence,
        }
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn reverse(&self) -> bool {
        self.reverse
    }

    pub fn silence(&self) -> bool {
        self.silence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_next() {
        let mut slice = SegmentSlice::new(
            vec![Segment::new(0, false, false), Segment::new(16, true, true)],
            1,
            0.,
        );
        assert_eq!(slice.next(), Some(Segment::new(0, false, false)));
        assert_eq!(slice.next(), Some(Segment::new(16, true, true)));
        assert_eq!(slice.next(), None);
    }

    #[test]
    fn layout_next() {
        let mut layout = SegmentLayout {
            segments: vec![
                vec![Segment::new(0, false, false), Segment::new(16, true, true)],
                vec![Segment::new(0, true, true), Segment::new(16, false, false)],
            ],
            index: 0,
            segment_len: 1,
        };

        let mut slices = vec![];
        slices.push(layout.next().unwrap());
        slices.push(layout.next().unwrap());

        assert_eq!(layout.next(), None);
        assert_eq!(slices[0].next(), Some(Segment::new(0, false, false)));
        assert_eq!(slices[0].next(), Some(Segment::new(0, true, true)));
        assert_eq!(slices[0].next(), None);
        assert_eq!(slices[1].next(), Some(Segment::new(16, true, true)));
        assert_eq!(slices[1].next(), Some(Segment::new(16, false, false)));
        assert_eq!(slices[1].next(), None);
    }

    #[test]
    fn segment_len_test() {
        assert_eq!(SegmentLayout::segment_len(48000, 120., 0.25), 24000);
    }

    #[test]
    fn segment_count_complete() {
        assert_eq!(SegmentLayout::segment_count(4000, 1000), (4, false));
    }

    #[test]
    fn segment_count_incomplete() {
        assert_eq!(SegmentLayout::segment_count(4001, 1000), (5, true));
    }

    #[test]
    fn channel_build_nothing() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.,
            prob_swap: 0.,
            prob_reverse: 0.,
            prob_repeat: 0.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let mut channel = SegmentLayout::build_channel(cli_config, wav_config).into_iter();

        assert_eq!(channel.next(), Some(Segment::new(0, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(3600, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(7200, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(10800, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(14400, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(18000, false, false)));
        assert_eq!(channel.next(), None);
    }

    #[test]
    fn channel_build_silence() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 1.,
            prob_swap: 0.,
            prob_reverse: 0.,
            prob_repeat: 0.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let mut channel = SegmentLayout::build_channel(cli_config, wav_config).into_iter();

        assert_eq!(channel.next(), Some(Segment::new(0, false, true)));
        assert_eq!(channel.next(), Some(Segment::new(3600, false, true)));
        assert_eq!(channel.next(), Some(Segment::new(7200, false, true)));
        assert_eq!(channel.next(), Some(Segment::new(10800, false, true)));
        assert_eq!(channel.next(), Some(Segment::new(14400, false, true)));
        assert_eq!(channel.next(), Some(Segment::new(18000, false, true)));
        assert_eq!(channel.next(), None);
    }

    #[test]
    fn channel_build_swap() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.,
            prob_swap: 1.,
            prob_reverse: 0.,
            prob_repeat: 0.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let mut channel = SegmentLayout::build_channel(cli_config, wav_config).into_iter();

        assert_eq!(channel.next(), Some(Segment::new(3600, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(7200, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(10800, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(14400, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(0, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(18000, false, false)));
        assert_eq!(channel.next(), None);
    }

    #[test]
    fn channel_build_reverse() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.,
            prob_swap: 0.,
            prob_reverse: 1.,
            prob_repeat: 0.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let mut channel = SegmentLayout::build_channel(cli_config, wav_config).into_iter();

        assert_eq!(channel.next(), Some(Segment::new(0, true, false)));
        assert_eq!(channel.next(), Some(Segment::new(3600, true, false)));
        assert_eq!(channel.next(), Some(Segment::new(7200, true, false)));
        assert_eq!(channel.next(), Some(Segment::new(10800, true, false)));
        assert_eq!(channel.next(), Some(Segment::new(14400, true, false)));
        assert_eq!(channel.next(), Some(Segment::new(18000, true, false)));
        assert_eq!(channel.next(), None);
    }

    #[test]
    fn channel_build_repeat() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.,
            prob_swap: 0.,
            prob_reverse: 0.,
            prob_repeat: 1.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let mut channel = SegmentLayout::build_channel(cli_config, wav_config).into_iter();

        assert_eq!(channel.next(), Some(Segment::new(0, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(0, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(7200, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(7200, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(14400, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(18000, false, false)));
        assert_eq!(channel.next(), None);
    }

    #[test]
    fn channel_build_swap_repeat() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.,
            prob_swap: 1.,
            prob_reverse: 0.,
            prob_repeat: 1.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let mut channel = SegmentLayout::build_channel(cli_config, wav_config).into_iter();

        assert_eq!(channel.next(), Some(Segment::new(3600, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(3600, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(10800, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(10800, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(0, false, false)));
        assert_eq!(channel.next(), Some(Segment::new(18000, false, false)));
        assert_eq!(channel.next(), None);
    }

    #[test]
    fn layout_build_same() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.,
            prob_swap: 0.,
            prob_reverse: 0.,
            prob_repeat: 0.,
            max_swap: 1,
            max_repeat: 1,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let layout = SegmentLayout::build(cli_config, wav_config);
        let mut channels = vec![vec![], vec![]];

        for slice in layout {
            for (i, segment) in slice.enumerate() {
                channels[i].push(segment);
            }
        }

        assert_eq!(channels[0], channels[1]);
    }

    #[test]
    fn layout_build_different() {
        let cli_config = CliConfig {
            tempo: 200.,
            segment_length: 0.0625,
            prob_silence: 0.5,
            prob_swap: 0.5,
            prob_reverse: 0.5,
            prob_repeat: 0.5,
            max_swap: 5,
            max_repeat: 5,
            each_channel_separately: true,
        };
        let wav_config = WavConfig {
            duration: 19800,
            sample_rate: 48000,
            channels: 2,
        };
        let layout = SegmentLayout::build(cli_config, wav_config);
        let mut channels = vec![vec![], vec![]];

        for slice in layout {
            for (i, segment) in slice.enumerate() {
                channels[i].push(segment);
            }
        }

        assert_ne!(channels[0], channels[1]);
    }
}
