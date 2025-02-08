#![no_main]

use libfuzzer_sys::fuzz_target;
use wavglitch::cli::{Cli, Parser};
use wavglitch::run;

fuzz_target!(
    |data: (u128, f32, u16, u16, f32, f32, f32, f32, u16, u16)| {
        let o = &format!("fuzz/res/{}", data.0);
        let t = if data.1.is_normal() { &(1. + data.1 % 4094.).to_string() } else { "10" };
        let l = &format!("{}/{}", data.2.clamp(1, 65535), data.3.clamp(1, 65535));
        let s = if data.4.is_normal() { &(data.4 % 1.).to_string() } else { "0.5" };
        let w = if data.5.is_normal() { &(data.5 % 1.).to_string() } else { "0.5" };
        let r = if data.6.is_normal() { &(data.6 % 1.).to_string() } else { "0.5" };
        let p = if data.7.is_normal() { &(data.7 % 1.).to_string() } else { "0.5" };
        let a = &(data.8.clamp(1, 65535)).to_string();
        let n = &(data.9.clamp(1, 65535)).to_string();
        if let Ok(cli) = Cli::try_parse_from(["fuzz", "fuzz/res/fuzz_in.wav", "-o", o, "-t", t, "-l", l, "-s", s, "-w", w, "-r", r, "-p", p, "-a", a, "-n", n, "-c"]) {
            let _ = std::fs::remove_file(o);
            assert_eq!(run::run(cli).unwrap(), ());
        }
    }
);
