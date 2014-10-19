#![feature(if_let)]
#![feature(phase)]

#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate getopts;
extern crate lines;

use std::os;
use std::io::{File, IoResult, BufferedReader};
use std::io::stdio::{stderr, stdout};

use getopts::{getopts, optflag};

use lines::linemapper;

use buf::MemWriter;

mod buf;

struct Config {
    line_numbers: bool,
    args: Vec<String>,
}

impl Config {
    fn from_cmdline(args: Vec<String>) -> Result<Config, String> {
        let opts = [
            optflag("n", "line-numbers", "print line numbers"),
            ];
        let matches = match getopts(args.tail(), opts) {
            Ok(m) => m,
            Err(f) => {
                return Err(format!("{}", f));
            }
        };
        if matches.free.len() < 1 {
            return Err(String::from_str("Usage: [OPTIONS] <logfile> [<logfile> ...]"));
        }
        Ok(Config {
            line_numbers: matches.opt_present("n"),
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

    let entry_start = regex!(r"^\d{4}-");

    let mut block = MemWriter::with_capacity(1024);
    let mut stdout = stdout().unwrap();

    let mut line_no = 0u;
    let mut collecting_block = false;
    linemapper::map_lines(r, |line| {
        line_no += 1;
        if entry_start.is_match(String::from_utf8_lossy(line).as_slice()) {
            if collecting_block {
                let _ = block.write_char('\n');
                // ~ abort as soon as possible if the write didn't
                // succeed e.g. broken pipe
                if let Err(e) = stdout.write(block.get_ref()) {
                    let _ = write!(stderr(), "{}", e);
                    std::os::set_exit_status(1);
                    return false;
                }
                collecting_block = false;
            }
            block.clear();
        } else {
            collecting_block = true;
        }
        if cfg.line_numbers {
            let _ = write!(block, "{:7u}  ", line_no);
        }
        let _ = block.write(line);
        true
    })
}
