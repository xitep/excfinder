#![feature(exit_status)]

extern crate regex;
extern crate getopts;
extern crate lines;

use std::env;
use std::fs::File;
use std::io::{BufReader, Write, stderr, stdout};
use std::io::Result as IoResult;

use getopts::Options;

use regex::Regex;
use lines::linemapper;

fn main() {
    let (cfg, files) = match Config::from_cmdline(env::args().collect()) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = stderr().write_all(e.as_ref());
            env::set_exit_status(1);
            return;
        }
    };
    for file in files {
        if let Err(e) = process_arg(&cfg, &file) {
            let _ = write!(&mut stderr(), "{}", e);
            env::set_exit_status(1);
            return;
        }
    }
}

// --------------------------------------------------------------------

struct Config {
    print_line_numbers: bool,
    print_filenames: bool,
    line_regex: Regex,
}

impl Config {
    fn from_cmdline(args: Vec<String>) -> Result<(Config, Vec<String>), String> {
        let mut opts = Options::new();
        opts.optflag("h", "help", "print this help screen");
        opts.optflag("n", "line-numbers", "print line numbers");
        opts.optflag("f", "filename", "print file name");
        opts.optopt("e", "entry", "identify regular lines", "regex");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => { return Err(format!("{}", f)); }
        };
        if matches.opt_present("h") || matches.free.len() < 1 {
            return Err(opts.usage("usage: [OPTIONS] <logfile> [<logfile> ...]"));
        }
        let re = {
            let s = matches.opt_str("e").unwrap_or_else(|| {
                r"^\d{4}-\d{2}-\d{2}\s+".to_owned()
            });
            match Regex::new(&s) {
                Ok(r) => r,
                Err(e) => { return Err(format!("{}", e)); }
            }
        };
        Ok((Config {
            print_line_numbers: matches.opt_present("n"),
            print_filenames: matches.opt_present("f"),
            line_regex: re
           },
           matches.free))
    }
}

// --------------------------------------------------------------------

const NEWLINE: &'static [u8] = &[b'\n'];
const SPACE: &'static [u8] = &[b' '];

// ~ XXX use linereader api and return the io error instead of
// handling it here in this function
fn process_arg(cfg: &Config, filename: &str) -> IoResult<()> {
    let f = try!(File::open(filename));
    let r = BufReader::new(f);

    let mut block = Vec::with_capacity(1024);
    let mut stdout = stdout();

    let mut line_no = 0usize;
    let mut collecting_block = false;
    let mut output_produced = false;
    linemapper::map_lines(r, |line| {
        line_no += 1;
        if cfg.line_regex.is_match(&String::from_utf8_lossy(line)) {
            if collecting_block {
                // ~ abort as soon as possible if the write didn't
                // succeed e.g. broken pipe
                if let Err(e) = stdout.write_all(&block) {
                    let _ = write!(&mut stderr(), "{}", e);
                    env::set_exit_status(1);
                    return false;
                }
                output_produced = true;
                collecting_block = false;
            }
            block.clear();

            if output_produced {
                let _ = block.write_all(NEWLINE);
            }
        } else {
            collecting_block = true;
        }

        // XXX prevent the buffer from becoming too large by flushing
        // it out as of a certain size

        if cfg.print_filenames {
            let _ = block.write_all(filename.as_ref());
            let _ = block.write_all(SPACE);
        }
        if cfg.print_line_numbers {
            let _ = write!(&mut block, "{:7} ", line_no);
        }
        let _ = block.write(line);
        true
    })
}
