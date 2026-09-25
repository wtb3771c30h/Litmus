//! `litmus` command-line interface.
//!
//! Usage:
//!
//! ```text
//! litmus run <scenario.litmus> [options]
//! litmus check <scenario.litmus> [options]   # like run, exits non-zero on violation
//! litmus emit <scenario.litmus> [--out FILE] # run and write JSON evidence
//! litmus explain <scenario.litmus>           # parse and print the scenario config
//! litmus version | help
//! ```
//!
//! Options:
//!   --runs N            override the scenario's run count
//!   --timeout-ms N      override the per-run timeout
//!   --seed N            override the base seed
//!   --json              print JSON evidence to stdout instead of a text report
//!   --out FILE          write JSON evidence to FILE (implies emit)
//!   --no-color          disable ANSI colors
//!   --sample-lines N    lines of sample output to show per group (default 4)

use std::process::ExitCode;

use litmus::report::investigate;
use litmus::{render_text, Scenario};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("litmus: error: {e}");
            ExitCode::from(2)
        }
    }
}

struct Options {
    scenario_path: String,
    runs: Option<u32>,
    timeout_ms: Option<u64>,
    seed: Option<u64>,
    json: bool,
    out: Option<String>,
    color: bool,
    sample_lines: usize,
}

enum Mode {
    Run,
    Check,
    Emit,
    Explain,
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let command = match args.first().map(String::as_str) {
        None | Some("help") | Some("-h") | Some("--help") => {
            print_help();
            return Ok(ExitCode::SUCCESS);
        }
        Some("version") | Some("-V") | Some("--version") => {
            println!("litmus {}", env!("CARGO_PKG_VERSION"));
            return Ok(ExitCode::SUCCESS);
        }
        Some("run") => Mode::Run,
        Some("check") => Mode::Check,
        Some("emit") => Mode::Emit,
        Some("explain") => Mode::Explain,
        Some(other) => return Err(format!("unknown command {other:?} (try `litmus help`)")),
    };

    let opts = parse_options(&args[1..])?;

    let source = std::fs::read_to_string(&opts.scenario_path)
        .map_err(|e| format!("cannot read {}: {e}", opts.scenario_path))?;
    let mut scenario = Scenario::parse(&source).map_err(|e| e.to_string())?;

    if let Some(r) = opts.runs {
        if r == 0 {
            return Err("--runs must be >= 1".to_string());
        }
        scenario.runs = r;
    }
    if let Some(t) = opts.timeout_ms {
        scenario.timeout_ms = Some(t);
    }
    if let Some(s) = opts.seed {
        scenario.seed = s;
    }

    if let Mode::Explain = command {
        print_explain(&scenario);
        return Ok(ExitCode::SUCCESS);
    }

    let report = investigate(&scenario).map_err(|e| format!("execution failed: {e}"))?;

    match command {
        Mode::Emit => {
            let json = report.to_json().to_pretty();
            match &opts.out {
                Some(path) => {
                    std::fs::write(path, &json).map_err(|e| format!("cannot write {path}: {e}"))?;
                    eprintln!("litmus: wrote evidence to {path}");
                }
                None => print!("{json}"),
            }
            Ok(ExitCode::SUCCESS)
        }
        Mode::Run | Mode::Check => {
            if opts.json {
                print!("{}", report.to_json().to_pretty());
            } else {
                print!("{}", render_text(&report, opts.color, opts.sample_lines));
            }
            if let Some(path) = &opts.out {
                std::fs::write(path, report.to_json().to_pretty())
                    .map_err(|e| format!("cannot write {path}: {e}"))?;
                eprintln!("litmus: wrote evidence to {path}");
            }
            if matches!(command, Mode::Check) && report.is_failure() {
                Ok(ExitCode::from(1))
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
        Mode::Explain => unreachable!("handled above"),
    }
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut scenario_path: Option<String> = None;
    let mut runs = None;
    let mut timeout_ms = None;
    let mut seed = None;
    let mut json = false;
    let mut out = None;
    let mut color = supports_color();
    let mut sample_lines = 4usize;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--runs" => {
                runs = Some(parse_num(next(args, &mut i, "--runs")?)?);
            }
            "--timeout-ms" => {
                timeout_ms = Some(parse_num(next(args, &mut i, "--timeout-ms")?)?);
            }
            "--seed" => {
                seed = Some(parse_num(next(args, &mut i, "--seed")?)?);
            }
            "--sample-lines" => {
                sample_lines = parse_num::<usize>(next(args, &mut i, "--sample-lines")?)?;
            }
            "--out" => {
                out = Some(next(args, &mut i, "--out")?.to_string());
            }
            "--json" => json = true,
            "--no-color" => color = false,
            "--color" => color = true,
            other if other.starts_with('-') => {
                return Err(format!("unknown option {other:?}"));
            }
            other => {
                if scenario_path.is_some() {
                    return Err(format!("unexpected extra argument {other:?}"));
                }
                scenario_path = Some(other.to_string());
            }
        }
        i += 1;
    }

    let scenario_path = scenario_path.ok_or("missing scenario file path")?;
    Ok(Options {
        scenario_path,
        runs,
        timeout_ms,
        seed,
        json,
        out,
        color,
        sample_lines,
    })
}

fn next<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    *i += 1;
    args.get(*i)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_num<T: std::str::FromStr>(s: &str) -> Result<T, String> {
    s.parse::<T>().map_err(|_| format!("invalid number: {s:?}"))
}

/// Respect `NO_COLOR` (https://no-color.org) and disable color when stdout is
/// likely not a terminal is hard without extra deps, so we default to on and
/// let `--no-color` / `NO_COLOR` turn it off.
fn supports_color() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}

fn print_explain(s: &Scenario) {
    println!("scenario     : {}", s.name);
    if !s.description.is_empty() {
        println!("description  : {}", s.description);
    }
    println!("command      : {}", s.command.join(" "));
    println!("runs         : {}", s.runs);
    println!(
        "timeout_ms   : {}",
        s.timeout_ms
            .map(|t| t.to_string())
            .unwrap_or_else(|| "none".into())
    );
    println!("seed         : {}", s.seed);
    println!("shuffle_env  : {}", s.shuffle_env);
    println!(
        "workdir      : {}",
        s.workdir.as_deref().unwrap_or("(inherited)")
    );
    println!("expect       : {}", s.expect);
    if !s.env.is_empty() {
        println!("env:");
        for (k, v) in &s.env {
            println!("  {k} = {v}");
        }
    }
    println!("normalize:");
    println!("  strip_timestamps = {}", s.normalize.strip_timestamps);
    println!("  strip_hex        = {}", s.normalize.strip_hex);
    println!("  strip_temp_paths = {}", s.normalize.strip_temp_paths);
    println!("  trim_trailing_ws = {}", s.normalize.trim_trailing_ws);
    println!("  sort_lines       = {}", s.normalize.sort_lines);
    if !s.normalize.drop_substrings.is_empty() {
        println!("  drop_substrings  = {:?}", s.normalize.drop_substrings);
    }
}

fn print_help() {
    println!(
        r#"litmus {ver} — a nondeterminism / flaky-test laboratory

USAGE:
    litmus <command> <scenario-file> [options]

COMMANDS:
    run       Run the scenario and print a report.
    check     Like `run`, but exit non-zero if the scenario's expectation
              (deterministic / flaky) is violated. Use this in CI.
    emit      Run the scenario and write JSON evidence (stdout or --out).
    explain   Parse the scenario and print its resolved configuration.
    version   Print the version.
    help      Show this help.

OPTIONS:
    --runs N            Override the scenario run count.
    --timeout-ms N      Override the per-run timeout (milliseconds).
    --seed N            Override the base seed.
    --json              Emit JSON evidence to stdout instead of a text report.
    --out FILE          Also write JSON evidence to FILE.
    --sample-lines N    Sample output lines shown per outcome group (default 4).
    --no-color          Disable ANSI color output (also honors NO_COLOR).

EXAMPLES:
    litmus run scenarios/deterministic-echo.litmus
    litmus check scenarios/flaky-timestamp.litmus
    litmus emit scenarios/flaky-timestamp.litmus --out evidence.json
"#,
        ver = env!("CARGO_PKG_VERSION")
    );
}
