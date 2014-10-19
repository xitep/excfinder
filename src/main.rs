#![feature(if_let)]

extern crate regex;
extern crate getopts;
extern crate lines;

use std::os;
use std::io::{File, IoResult, BufferedReader};
use std::io::stdio::{stderr, stdout};

use getopts::{getopts, optopt, optflag, usage};

use regex::Regex;
use lines::linemapper;

use buf::MemWriter;

mod buf;

struct Config {
    print_line_numbers: bool,
    print_filenames: bool,
    line_regex: Regex,
    args: Vec<String>,
}

impl Config {
    fn from_cmdline(args: Vec<String>) -> Result<Config, String> {
        let opts = [
            optflag("h", "help", "print this help screen"),
            optflag("n", "line-numbers", "print line numbers"),
            optflag("f", "filename", "print file name"),
            optopt("e", "entry", "identify regular lines", "regex"),
            ];
        let matches = match getopts(args.tail(), opts) {
            Ok(m) => m,
            Err(f) => {
                return Err(format!("{}", f));
            }
        };
        if matches.opt_present("h") || matches.free.len() < 1 {
            return Err(usage("usage: [OPTIONS] <logfile> [<logfile> ...]", opts));
        }
        let re = {
            let s = matches.opt_str("e").unwrap_or_else(|| {
                String::from_str(r"^\d{4}-\d{2}-\d{2}\s+")
            });
            match Regex::new(s.as_slice()) {
                Ok(r) => r,
                Err(e) => {
                    return Err(format!("{}", e));
                }
            }
        };
        Ok(Config {
            print_line_numbers: matches.opt_present("n"),
            print_filenames: matches.opt_present("f"),
            line_regex: re,
            args: matches.free,
        })
    }
}

// --------------------------------------------------------------------

fn main() {
    let cfg = match Config::from_cmdline(os::args()) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = stderr().write_line(e.as_slice());
            std::os::set_exit_status(1);
            return;
        }
    };
    for arg in cfg.args.iter() {
        if let Err(e) = process_arg(&cfg, arg.as_slice()) {
            let _ = write!(stderr(), "{}", e);
            std::os::set_exit_status(1);
            return;
        }
    }
}

// ~ XXX use linereader api and return the io error instead of
// handling it here in this function
fn process_arg(cfg: &Config, arg: &str) -> IoResult<()> {
    let f = try!(File::open(&Path::new(arg)));
    let r = BufferedReader::new(f);

    let mut block = MemWriter::with_capacity(1024);
    let mut stdout = stdout().unwrap();

    let mut line_no = 0u;
    let mut collecting_block = false;
    let mut output_produced = false;
    linemapper::map_lines(r, |line| {
        line_no += 1;
        if cfg.line_regex.is_match(String::from_utf8_lossy(line).as_slice()) {
            if collecting_block {
                // ~ abort as soon as possible if the write didn't
                // succeed e.g. broken pipe
                if let Err(e) = stdout.write(block.get_ref()) {
                    let _ = write!(stderr(), "{}", e);
                    std::os::set_exit_status(1);
                    return false;
                }
                output_produced = true;
                collecting_block = false;
            }
            block.clear();

            if output_produced {
                let _ = block.write_char('\n');
            }
        } else {
            collecting_block = true;
        }

        // XXX prevent the buffer from becoming too large by flushing
        // it out as of a certain size

        if cfg.print_filenames {
            let _ = block.write_str(arg);
            let _ = block.write_str("  ");
        }
        if cfg.print_line_numbers {
            let _ = write!(block, "{:7u}  ", line_no);
        }
        let _ = block.write(line);
        true
    })
}
