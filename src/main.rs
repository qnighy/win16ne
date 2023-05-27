use std::fs::File;
use std::io::{self, BufReader, Cursor, Read};
use std::path::PathBuf;
use clap::Parser;

pub mod mz;
pub mod ne;
pub mod x86;

use ne::NeExecutable;

#[derive(Debug, Clone, Parser)]
pub struct Opts {
    #[clap(short, long)]
    disassemble: bool,

    #[clap(long)]
    data: bool,

    #[clap(name = "FILE", value_parser)]
    files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    env_logger::init();

    let opts = Opts::parse();

    if opts.files.is_empty() {
        eprintln!("Error: no files specified");
        std::process::exit(1);
    }

    for file in &opts.files {
        let data = {
            let mut f = BufReader::new(File::open(file)?);
            let mut data = Vec::new();
            f.read_to_end(&mut data)?;
            data
        };

        let mut cursor = Cursor::new(data.as_slice());

        let parsed = NeExecutable::read(&mut cursor)?;
        parsed.describe(opts.data, opts.disassemble);
    }
    Ok(())
}
