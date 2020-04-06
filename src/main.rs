// Copyright 2020 Google LLC
//
// Use of this source code is governed by an MIT-style license that can be found
// in the LICENSE file or at https://opensource.org/licenses/MIT.

use std::fs::File;
use std::io::Result;
use std::time::Duration;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Verbosity(log::LevelFilter);

impl std::str::FromStr for Verbosity {

    type Err = String; // TODO.

    fn from_str(string: &str) -> std::result::Result<Verbosity, String> {
        use log::LevelFilter::*;

        match string {
            "quiet" => Ok(Verbosity(Off)),
            "error" => Ok(Verbosity(Error)),
            "warn" => Ok(Verbosity(Warn)),
            "info" => Ok(Verbosity(Info)),
            "debug" => Ok(Verbosity(Debug)),
            "trace" => Ok(Verbosity(Trace)),
            _ => Err(format!("invalid verbosity choice '{}'", string)),
        }
    }
}

// TODO: This should just be a wrapper around `simplelog::TerminalMode`, but
// it does not implement standard traits. So, for now, we just re-implement it
// like that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Std {
    Out,
    Err,
    Mix,
}

impl std::str::FromStr for Std {

    type Err = String; // TODO.

    fn from_str(string: &str) -> std::result::Result<Std, String> {
        match string {
            "out" => Ok(Std::Out),
            "err" => Ok(Std::Err),
            "mix" => Ok(Std::Mix),
            _ => Err(format!("invalid std choice '{}'", string)),
        }
    }
}

#[derive(structopt_derive::StructOpt)]
#[structopt(name = "RRG", about = "A GRR agent rewritten in Rust.")]
struct Opts {
    #[structopt(long="log-verbosity", name="LEVEL", default_value="info",
                help="A log verbosity level")]
    log_verbosity: Verbosity,

    #[structopt(long="log-std", name="STD",
                help="A standard stream to log to")]
    log_std: Option<Std>,

    #[structopt(long="log-file", name="FILE",
                help="A file to log to")]
    log_file: Option<PathBuf>,

    #[structopt(long="heartbeat-rate", name="DURATION", default_value="5s",
                parse(try_from_str = "humantime::parse_duration"),
                help="A frequency of Fleetspeak heartbeat messages.")]
    heartbeat_rate: Duration,
}

fn main() -> Result<()> {
    let opts = <Opts as structopt::StructOpt>::from_args();
    init(&opts);

    fleetspeak::startup(env!("CARGO_PKG_VERSION"))?;

    loop {
        let packet = fleetspeak::collect(opts.heartbeat_rate)?;
        handle(packet.data);
    }
}

fn init(opts: &Opts) {
    let Verbosity(level) = opts.log_verbosity;
    let mut loggers = Vec::<Box<dyn simplelog::SharedLogger>>::new();

    if let Some(std) = &opts.log_std {
        let config = Default::default();

        use simplelog::TerminalMode::*;
        let logger = simplelog::TermLogger::new(level, config, match std {
            Std::Out => Stdout,
            Std::Err => Stderr,
            Std::Mix => Mixed,
        }).expect("failed to create a terminal logger");

        loggers.push(logger);
    }

    if let Some(path) = &opts.log_file {
        let config = Default::default();

        let file = File::create(path).expect("failed to create the log file");
        let logger = simplelog::WriteLogger::new(level, config, file);

        loggers.push(logger);
    }

    simplelog::CombinedLogger::init(loggers).expect("failed to init logging");
}

fn handle(message: rrg_proto::GrrMessage) {
    match message.name {
        Some(name) => println!("requested to execute the '{}' action", name),
        None => eprintln!("missing action name to execute"),
    }
}
