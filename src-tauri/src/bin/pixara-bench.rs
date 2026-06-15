//! CLI for headless conversion benchmarks (Phase 0).
//!
//! Build: `cargo build --release --bin pixara-bench`
//! HEIC: copy vcpkg DLLs next to pixara-bench.exe (see scripts/bench-convert.ps1).

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use pixara_lib::bench::{self, BenchResult};

fn usage() -> &'static str {
    r#"pixara-bench — headless batch conversion timing

Usage:
  pixara-bench --input <dir> --output <dir> [options]

Options:
  --format <png|jpeg|webp|heic|...>   Target format (default: png)
  --preset <web|high|smallest>        Quality preset (default: web)
  --workers <n>                       Rayon threads (0 = auto, default: 0)
  --optimize-png <true|false>         oxipng pass for PNG (default: true)
  --help                              Show this help

Prints one JSON line to stdout (BenchResult, includes stageMs / stagePct).
"#
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", usage());
        return ExitCode::SUCCESS;
    }

    let input = match arg_value(&args, "--input") {
        Some(v) => PathBuf::from(v),
        None => {
            eprintln!("missing --input");
            eprint!("{}", usage());
            return ExitCode::from(2);
        }
    };
    let output = match arg_value(&args, "--output") {
        Some(v) => PathBuf::from(v),
        None => {
            eprintln!("missing --output");
            return ExitCode::from(2);
        }
    };

    let format_s = arg_value(&args, "--format").unwrap_or_else(|| "png".into());
    let preset_s = arg_value(&args, "--preset").unwrap_or_else(|| "web".into());
    let workers_s = arg_value(&args, "--workers").unwrap_or_else(|| "0".into());
    let optimize_s = arg_value(&args, "--optimize-png").unwrap_or_else(|| "true".into());

    let to_format = match bench::parse_format(&format_s) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let preset = match bench::parse_preset(&preset_s) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let workers: Option<usize> = match workers_s.parse::<usize>() {
        Ok(0) => None,
        Ok(n) => Some(n),
        Err(_) => {
            eprintln!("invalid --workers: {workers_s}");
            return ExitCode::from(2);
        }
    };
    let optimize_png = matches!(
        optimize_s.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    );

    let settings = bench::bench_settings(to_format, preset, &output, optimize_png);

    match bench::run_bench(&input, &output, settings, workers) {
        Ok(result) => {
            if result.failed > 0 {
                eprintln!(
                    "warning: {}/{} conversions failed",
                    result.failed, result.files
                );
            }
            print_result(&result);
            if result.failed > 0 {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("bench failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn print_result(result: &BenchResult) {
    match serde_json::to_string(result) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("serialize: {e}"),
    }
}
