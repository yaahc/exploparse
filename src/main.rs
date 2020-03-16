#![feature(backtrace)]

extern crate csv;

// this result is different. While normally result is an enum, this returns a <T> or an error which is muted.
use csv::StringRecord;
use serde::Deserialize;
use tracing_subscriber::{prelude::*, registry::Registry};
use tracing_error::ErrorLayer;
use spandoc::spandoc;

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "LC")]
    lc: String,
}

#[spandoc]
fn main() -> Result<(), exploparse::ErrReport> {
    color_backtrace::install();

    let subscriber = Registry::default()
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

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

        /// Normalizing first field of csv data rows
        match exploparse::LC::maybe_parse(lc) {
            Ok(Some(lc @ exploparse::LC { note: None, .. })) => {
                let mut new_record = StringRecord::new();
                new_record.push_field(&lc.to_string());
                new_record.extend(record.iter().skip(1));
                writer.write_record(&new_record)?;
            }
            Ok(Some(_)) => questionable_rows.push(record),
            Ok(None) => bad_rows.push(record),
            Err(e) => {
                eprintln!("Error: {:?}\n", e);
                bad_rows.push(record);
            }
        }
    }

    for record in questionable_rows {
        writer.write_record(&record)?;
    }

    for record in bad_rows {
        writer.write_record(&record)?;
    }

    Ok(())
}
