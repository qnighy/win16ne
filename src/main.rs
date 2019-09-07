use std::fs::File;
use std::io::{self, BufReader, Cursor, Read};

pub mod mz;
pub mod ne;

use ne::NeExecutable;

fn main() -> io::Result<()> {
    env_logger::init();

    let data = {
        let mut f = BufReader::new(File::open("a.exe")?);
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        data
    };

    let mut cursor = Cursor::new(data.as_slice());

    let parsed = NeExecutable::read(&mut cursor)?;
    parsed.describe();
    Ok(())
}
