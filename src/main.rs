#![feature(backtrace)]

use jane_eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "LC")]
    lc: String,
}

fn main() -> Result<()> {
    let mut reader = csv::Reader::from_path("./exploLib.csv")?;
    let reader = reader.deserialize();

    for result in reader {
        let row: Row = result?;
        println!("{:?}", row);
        let lc = exploparse::LC::maybe_parse(row.lc.trim()).unwrap();
        println!("{:?}", lc);
    }

    Ok(())
}
