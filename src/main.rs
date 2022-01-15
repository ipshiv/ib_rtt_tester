#![feature(option_result_contains)]

use chrono::{DateTime, Datelike, Timelike, Utc};
use clap::{App, Arg};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result};

fn test_line(cmd: &str) {
    if cmd.contains("* `") {
        println!("CMD {}", cmd)
    } else {
        println!("{}", cmd);
    }
}

fn main() -> Result<()> {
    let matches = App::new("ib_rtt_tester")
        .arg(
            Arg::with_name("test_case")
                .takes_value(true)
                .help("Test case in markdown format"),
        )
        .arg(
            Arg::with_name("output_folder")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("Test run output folder"),
        )
        .arg(Arg::with_name("silent").short("s").long("silent"))
        .get_matches();

    let infile = matches.value_of("test_case").unwrap_or_default();

    /* Test source file - filename extension */
    let source_to_test = infile.split('/').last().unwrap();
    if !source_to_test.to_string().contains(&"t.md") {
        let e = Error::new(ErrorKind::InvalidData, "Test run wasn't found");
        return Err(e);
    }
    /* Test source file - file existence */
    let test_content_buffer = BufReader::new(File::open(infile)?);
    let dt: DateTime<Utc> = Utc::now();
    let outfile = format!(
        "{}-{}-{}-{:02}{}{}-",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ) + &*source_to_test.replace("t.md", "r.md");
    let silent = if matches.is_present("silent") {
        true
    } else {
        !env::var("PV_SILENT").unwrap_or_default().is_empty()
    };
    dbg!(infile, outfile, silent);
    /* Parse infile and search for test cases */
    test_content_buffer
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|x| test_line(&x));

    Ok(())
}
