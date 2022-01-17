#![feature(option_result_contains)]

use chrono::{DateTime, Datelike, Timelike, Utc};
use clap::{App, Arg};
use std::fs::File;
use std::io::{self, BufWriter, Error, ErrorKind, Result, Write};
use std::{env, fs};

struct Tester {
    log_backend: Box<dyn Write>,
    is_silent: bool,
}

impl Tester {
    fn init(is_silent: bool, outfile: &str) -> Tester {
        Tester {
            log_backend: if outfile.is_empty() {
                Box::new(BufWriter::new(io::stdout()))
            } else {
                Box::new(BufWriter::new(File::create(outfile).unwrap()))
            },
            is_silent,
        }
    }

    fn test_line(&mut self, cmd: &str) -> String {
        let mut string: String = "".to_string();
        if !cmd.contains("=>") {
            string.push_str(cmd);
            string.push_str(" :red_circle:\n");
        } else {
            let vec: Vec<&str> = cmd.split("=>").collect();
            string = vec[0].to_string();
            string.push_str(" :heavy_check_mark:\n");
        }

        if !self.is_silent {
            let log = string.as_str();
            println!("LOG: {}", log);
        }

        string.to_string()
    }

    fn test_section(&mut self, section: &str) -> Result<()> {
        let mut lines = section.lines();
        let sections_str = lines.next().unwrap();
        let mut result_str = "".to_string();
        println!("Section str {}\n", sections_str);
        let mut test_runs: String = "\n".to_owned();
        for line in lines.filter(|line| line.contains("* `")) {
            test_runs.push_str(self.test_line(line).as_str());
        }

        if test_runs.contains(":heavy_check_mark:") && !test_runs.contains(":red_circle:") {
            result_str.push_str(
                format!("* [success] {}", sections_str.replace('*', " ").as_str()).as_str(),
            )
        } else {
            result_str
                .push_str(format!("* [fail] {}", sections_str.replace('*', " ").as_str()).as_str())
        }
        result_str.push_str(test_runs.as_str());
        self.log_backend.write_all(result_str.as_bytes())?;
        Ok(())
    }

    fn run_test(&mut self, infile: &str) -> Result<()> {
        let test_content_buffer = fs::read_to_string(infile)?;
        let sections: Vec<_> = test_content_buffer.match_indices("\n*").collect();
        let mut prev_section_end: usize = sections[0].0 + 1 /*skip \n char */;

        /* skip header */
        self.log_backend.write_fmt(format_args!(
            "{}\n",
            &test_content_buffer[..prev_section_end]
        ))?;
        for section in &sections[1..] {
            self.test_section(&test_content_buffer[prev_section_end..section.0])?;
            prev_section_end = section.0 + 1;
        }
        self.test_section(&test_content_buffer[prev_section_end..])?;

        Ok(())
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

    /* Parse infile and search for test cases */
    let mut tester = Tester::init(silent, outfile.as_str());
    tester.run_test(infile)
}
