#![feature(backtrace)]
// this result is different. While normally result is an enum, this returns a <T> or an error which is muted. 
use jane_eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "LC")]
    lc: String,
}

fn main() -> Result<()> {
    let mut reader = csv::Reader::from_path("./exploLibMain.csv")?;
    let reader = reader.deserialize();

    for result in reader {
        let row: Row = result?;
        println!("{:?}", row);
        let lc = exploparse::LC::maybe_parse(row.lc.trim()).unwrap();
        println!("{:?}", lc);
    }

    // idiosyncracy of rust
    Ok(())
}
