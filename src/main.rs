use std::fs::File;
use std::io::{self, BufReader, Cursor, Read};
use std::path::PathBuf;
use structopt::StructOpt;

pub mod mz;
pub mod ne;
pub mod x86;

use ne::NeExecutable;

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long)]
    disassemble: bool,

    #[structopt(long)]
    data: bool,

    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    env_logger::init();

    let opts = Opts::from_args();

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
