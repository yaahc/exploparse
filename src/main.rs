#![feature(backtrace)]

extern crate csv;

// this result is different. While normally result is an enum, this returns a <T> or an error which is muted.
use csv::StringRecord;
use jane_eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "LC")]
    lc: String,
}

fn main() -> Result<()> {
    let mut reader = csv::Reader::from_path("./exploLibMain.csv")?;
    let mut writer = csv::Writer::from_path("./exploLibOut.csv")?;
    let mut bad_rows = vec![];
    let mut questionable_rows = vec![];
    let header = reader.headers()?.clone();
    writer.write_record(&header)?;
    let records = reader.records();

    for result in records {
        let record = result?;
        let row: Row = record.deserialize(Some(&header))?;
        let lc = row.lc.trim();

        match exploparse::LC::maybe_parse(lc) {
            Ok(Some(lc @ exploparse::LC { note: None, ..})) => {
                let mut new_record = StringRecord::new();
                new_record.push_field(&lc.to_string());
                new_record.extend(record.iter().skip(1));
                writer.write_record(&new_record)?;
            }
            Ok(Some(_)) => questionable_rows.push(record),
            _ => bad_rows.push(record),
        }
    }

    for record in questionable_rows {
        writer.write_record(&record)?;
    }

    for record in bad_rows {
        writer.write_record(&record)?;
    }

    // idiosyncracy of rust
    Ok(())
}
