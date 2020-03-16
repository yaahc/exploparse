#![feature(backtrace)]

extern crate csv;

// this result is different. While normally result is an enum, this returns a <T> or an error which is muted. 
use jane_eyre::Result;
use serde::Deserialize;
use csv::Writer;

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(rename = "LC")]
    lc: String,
        // All columns from original file
        // Read and turn into fields for each entry then reprint
        // There are 30 total columns including LC (lc0, but too lazy to rename)
    #[serde(rename = "Accession Number")]   	
    lc1: String,
    #[serde(rename = "Author, Primary")]
    lc2: String,	
    #[serde(rename = "Authors, Multiple")]
    lc3: String,	
    #[serde(rename = "Barcode")]
    lc4: String,
    #[serde(rename = "Bibliographic Format")]	
    lc5: String,
    #[serde(rename = "Branch")]
    lc6: String,
    #[serde(rename = "Collection/Status")]
    lc7: String,
    #[serde(rename = "Copy")]
    lc8: String,
    #[serde(rename = "Corporate Author")]
    lc9: String,
    #[serde(rename = "Cost")]
    lc10: String,
    #[serde(rename = "Date Touched")]
    lc11: String,
    #[serde(rename = "Description")]	
    lc12: String,
    #[serde(rename = "Edition")]
    lc13: String,
    #[serde(rename = "General Material Designation")]
    lc14: String,
    #[serde(rename = "Home Branch")]
    lc15: String,
    #[serde(rename = "ISBN")]
    lc16: String,
    #[serde(rename = "ISBN-Normal")]
    lc17: String,	
    #[serde(rename = "ISBN-Normal 10")]
    lc18: String,
    #[serde(rename = "LC Classification 1")]	
    lc19: String,
    #[serde(rename = "LCCN")]
    lc20: String,
    #[serde(rename = "Place of Publication")]
    lc21: String,
    #[serde(rename = "Publication Dates")]
    lc22: String,
    #[serde(rename = "Publication Year")]
    lc23: String,
    #[serde(rename = "Publisher")]
    lc24: String,
    #[serde(rename = "Series")]
    lc25: String,
    #[serde(rename = "Status")]
    lc26: String,
    #[serde(rename = "Subject Headings, LC")]
    lc27: String,
    #[serde(rename = "Title")]
    lc28: String,
    #[serde(rename = "Titles, Alternate")]
    lc29: String,
}

fn main() -> Result<()> {
    let mut reader = csv::Reader::from_path("./exploLibMain.csv")?;
    let reader = reader.deserialize();

    for result in reader {
        let row: Row = result?;
        //println!("{:?}", row);
        // let lc = exploparse::LC::maybe_parse(row.lc.trim()).unwrap();
        // println!("{:?}", lc);

    match exploparse::LC::maybe_parse(row.lc.trim()) {
        Ok(lc) => { // your code here
            println!("{:?}", lc);
            //println!("{:?}", row);

            // // write out csv of non-error
            // // Still in work - doesn't write to csv yet
            // let write_csv = || {
            //     let mut wtr = Writer::from_path("newExplo.csv")?;
            //     wtr.write_record(row.unwrap().to_string())?;
            //     wtr.flush()?;
            //     Ok(())
            // };
        },
        Err(e) => { // your error handling code here
            
            println!("Row {:?} encountered error: {:?}", row, e);
        },
      }};

    // idiosyncracy of rust
    Ok(())
}
