use crate::segment_layout::SegmentSlice;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct WavReader {
    reader: hound::WavReader<BufReader<File>>,
}

#[derive(Copy, Clone)]
pub struct WavConfig {
    pub duration: u32,
    pub sample_rate: u32,
    pub channels: u16,
}

impl WavReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<WavReader> {
        let reader = hound::WavReader::open(path).context("when opening input file")?;
        Ok(WavReader { reader })
    }

    pub fn config(&self) -> WavConfig {
        let hound::WavSpec {
            channels,
            sample_rate,
            ..
        } = self.reader.spec();
        let duration = self.reader.duration();

        WavConfig {
            duration,
            sample_rate,
            channels,
        }
    }

    pub fn spec(&self) -> hound::WavSpec {
        self.reader.spec()
    }

    pub fn read<S: hound::Sample + std::clone::Clone + From<i16>>(
        &mut self,
        slice: &mut SegmentSlice,
    ) -> Result<Vec<S>> {
        let mut channels: Vec<Vec<S>> = vec![];
        let segment_len = slice.segment_len();

        for (i, segment) in slice.enumerate() {
            if segment.silence() {
                let channel = vec![
                    0.into();
                    (self.reader.duration() as usize - segment.offset() as usize)
                        .min(segment_len)
                ];
                channels.push(channel);
            } else {
                let mut channel = self
                    .read_segment(i, segment_len, segment.offset())
                    .context("when reading from input file")?;

                if segment.reverse() {
                    channel.reverse();
                }
                channels.push(channel);
            }
        }

        let mut samples: Vec<S> = vec![];

        for i in 0..channels[0].len() {
            for channel in &channels {
                samples.push(channel[i].clone());
            }
        }

        Ok(samples)
    }

    fn read_segment<S: hound::Sample>(
        &mut self,
        channel_idx: usize,
        segment_len: usize,
        segment_offset: u32,
    ) -> Result<Vec<S>, hound::Error> {
        let channel_count = self.reader.spec().channels as usize;
        self.reader.seek(segment_offset)?;

        self.reader
            .samples::<S>()
            .take(channel_count * segment_len)
            .enumerate()
            .filter(|(j, _)| j % channel_count == channel_idx)
            .map(|(_, s)| s)
            .collect::<Result<Vec<S>, hound::Error>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment_layout::Segment;
    use assert_fs::fixture::TempDir;
    use assert_fs::prelude::*;

    #[test]
    fn wav_reader_red_segment() {
        let dir = TempDir::new().unwrap();
        let input = dir.child("in.wav");
        input.write_binary(b"\x52\x49\x46\x46\x6c\x00\x00\x00\x57\x41\x56\x45\x66\x6d\x74\x20\x28\x00\x00\x00\
                             \xfe\xff\x02\x00\x80\xbb\x00\x00\x00\x65\x04\x00\x06\x00\x18\x00\x16\x00\x18\x00\
                             \x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71\
                             \x64\x61\x74\x61\x30\x00\x00\x00\x01\x00\x00\xff\xff\xff\x02\x00\x00\xfe\xff\xff\
                             \x03\x00\x00\xfd\xff\xff\x0b\x00\x00\xf5\xff\xff\x0c\x00\x00\xf4\xff\xff\x0d\x00\
                             \x00\xf3\xff\xff\x15\x00\x00\xeb\xff\xff\x16\x00\x00\xea\xff\xff").unwrap();
        let mut reader = WavReader::open(input).unwrap();

        assert_eq!(reader.read_segment::<i32>(0, 3, 0).unwrap(), [1, 2, 3]);
        assert_eq!(reader.read_segment::<i32>(0, 3, 3).unwrap(), [11, 12, 13]);
        assert_eq!(reader.read_segment::<i32>(0, 3, 6).unwrap(), [21, 22]);
        assert_eq!(reader.read_segment::<i32>(1, 3, 0).unwrap(), [-1, -2, -3]);
        assert_eq!(
            reader.read_segment::<i32>(1, 3, 3).unwrap(),
            [-11, -12, -13]
        );
        assert_eq!(reader.read_segment::<i32>(1, 3, 6).unwrap(), [-21, -22]);

        dir.close().unwrap();
    }

    #[test]
    fn wav_reader_read_1() {
        let dir = TempDir::new().unwrap();
        let input = dir.child("in.wav");
        input.write_binary(b"\x52\x49\x46\x46\x6c\x00\x00\x00\x57\x41\x56\x45\x66\x6d\x74\x20\x28\x00\x00\x00\
                             \xfe\xff\x02\x00\x80\xbb\x00\x00\x00\x65\x04\x00\x06\x00\x18\x00\x16\x00\x18\x00\
                             \x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71\
                             \x64\x61\x74\x61\x30\x00\x00\x00\x01\x00\x00\xff\xff\xff\x02\x00\x00\xfe\xff\xff\
                             \x03\x00\x00\xfd\xff\xff\x0b\x00\x00\xf5\xff\xff\x0c\x00\x00\xf4\xff\xff\x0d\x00\
                             \x00\xf3\xff\xff\x15\x00\x00\xeb\xff\xff\x16\x00\x00\xea\xff\xff").unwrap();
        let mut reader = WavReader::open(input).unwrap();
        let mut slice_1 = SegmentSlice::new(
            vec![Segment::new(0, false, false), Segment::new(0, false, false)],
            3,
            0.,
        );
        let mut slice_2 = SegmentSlice::new(
            vec![Segment::new(3, false, false), Segment::new(3, false, false)],
            3,
            0.,
        );
        let mut slice_3 = SegmentSlice::new(
            vec![Segment::new(6, false, false), Segment::new(6, false, false)],
            3,
            0.,
        );

        assert_eq!(
            reader.read::<i32>(&mut slice_1).unwrap(),
            [1, -1, 2, -2, 3, -3]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_2).unwrap(),
            [11, -11, 12, -12, 13, -13]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_3).unwrap(),
            [21, -21, 22, -22]
        );

        dir.close().unwrap();
    }

    #[test]
    fn wav_reader_read_2() {
        let dir = TempDir::new().unwrap();
        let input = dir.child("in.wav");
        input.write_binary(b"\x52\x49\x46\x46\x6c\x00\x00\x00\x57\x41\x56\x45\x66\x6d\x74\x20\x28\x00\x00\x00\
                             \xfe\xff\x02\x00\x80\xbb\x00\x00\x00\x65\x04\x00\x06\x00\x18\x00\x16\x00\x18\x00\
                             \x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71\
                             \x64\x61\x74\x61\x30\x00\x00\x00\x01\x00\x00\xff\xff\xff\x02\x00\x00\xfe\xff\xff\
                             \x03\x00\x00\xfd\xff\xff\x0b\x00\x00\xf5\xff\xff\x0c\x00\x00\xf4\xff\xff\x0d\x00\
                             \x00\xf3\xff\xff\x15\x00\x00\xeb\xff\xff\x16\x00\x00\xea\xff\xff").unwrap();
        let mut reader = WavReader::open(input).unwrap();
        let mut slice_1 = SegmentSlice::new(
            vec![Segment::new(0, false, true), Segment::new(0, false, true)],
            3,
            0.,
        );
        let mut slice_2 = SegmentSlice::new(
            vec![Segment::new(3, false, true), Segment::new(3, false, true)],
            3,
            0.,
        );
        let mut slice_3 = SegmentSlice::new(
            vec![Segment::new(6, false, true), Segment::new(6, false, true)],
            3,
            0.,
        );

        assert_eq!(
            reader.read::<i32>(&mut slice_1).unwrap(),
            [0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_2).unwrap(),
            [0, 0, 0, 0, 0, 0]
        );
        assert_eq!(reader.read::<i32>(&mut slice_3).unwrap(), [0, 0, 0, 0]);

        dir.close().unwrap();
    }

    #[test]
    fn wav_reader_read_3() {
        let dir = TempDir::new().unwrap();
        let input = dir.child("in.wav");
        input.write_binary(b"\x52\x49\x46\x46\x6c\x00\x00\x00\x57\x41\x56\x45\x66\x6d\x74\x20\x28\x00\x00\x00\
                             \xfe\xff\x02\x00\x80\xbb\x00\x00\x00\x65\x04\x00\x06\x00\x18\x00\x16\x00\x18\x00\
                             \x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71\
                             \x64\x61\x74\x61\x30\x00\x00\x00\x01\x00\x00\xff\xff\xff\x02\x00\x00\xfe\xff\xff\
                             \x03\x00\x00\xfd\xff\xff\x0b\x00\x00\xf5\xff\xff\x0c\x00\x00\xf4\xff\xff\x0d\x00\
                             \x00\xf3\xff\xff\x15\x00\x00\xeb\xff\xff\x16\x00\x00\xea\xff\xff").unwrap();
        let mut reader = WavReader::open(input).unwrap();
        let mut slice_1 = SegmentSlice::new(
            vec![Segment::new(0, true, false), Segment::new(0, true, false)],
            3,
            0.,
        );
        let mut slice_2 = SegmentSlice::new(
            vec![Segment::new(3, true, false), Segment::new(3, true, false)],
            3,
            0.,
        );
        let mut slice_3 = SegmentSlice::new(
            vec![Segment::new(6, true, false), Segment::new(6, true, false)],
            3,
            0.,
        );

        assert_eq!(
            reader.read::<i32>(&mut slice_1).unwrap(),
            [3, -3, 2, -2, 1, -1]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_2).unwrap(),
            [13, -13, 12, -12, 11, -11]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_3).unwrap(),
            [22, -22, 21, -21]
        );

        dir.close().unwrap();
    }

    #[test]
    fn wav_reader_read_4() {
        let dir = TempDir::new().unwrap();
        let input = dir.child("in.wav");
        input.write_binary(b"\x52\x49\x46\x46\x6c\x00\x00\x00\x57\x41\x56\x45\x66\x6d\x74\x20\x28\x00\x00\x00\
                             \xfe\xff\x02\x00\x80\xbb\x00\x00\x00\x65\x04\x00\x06\x00\x18\x00\x16\x00\x18\x00\
                             \x03\x00\x00\x00\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71\
                             \x64\x61\x74\x61\x30\x00\x00\x00\x01\x00\x00\xff\xff\xff\x02\x00\x00\xfe\xff\xff\
                             \x03\x00\x00\xfd\xff\xff\x0b\x00\x00\xf5\xff\xff\x0c\x00\x00\xf4\xff\xff\x0d\x00\
                             \x00\xf3\xff\xff\x15\x00\x00\xeb\xff\xff\x16\x00\x00\xea\xff\xff").unwrap();
        let mut reader = WavReader::open(input).unwrap();
        let mut slice_1 = SegmentSlice::new(
            vec![Segment::new(0, false, false), Segment::new(3, false, false)],
            3,
            0.,
        );
        let mut slice_2 = SegmentSlice::new(
            vec![Segment::new(3, false, false), Segment::new(0, false, false)],
            3,
            0.,
        );
        let mut slice_3 = SegmentSlice::new(
            vec![Segment::new(6, false, false), Segment::new(6, false, false)],
            3,
            0.,
        );

        assert_eq!(
            reader.read::<i32>(&mut slice_1).unwrap(),
            [1, -11, 2, -12, 3, -13]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_2).unwrap(),
            [11, -1, 12, -2, 13, -3]
        );
        assert_eq!(
            reader.read::<i32>(&mut slice_3).unwrap(),
            [21, -21, 22, -22]
        );

        dir.close().unwrap();
    }
}
