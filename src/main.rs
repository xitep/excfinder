extern crate regex;
extern crate getopts;

#[macro_use(try_read_lines)]
extern crate lines;

use std::env;
use std::fs::File;
use std::io::{Write, stderr, stdout};
use std::io::Result as IoResult;
use std::process;

use getopts::Options;

use regex::Regex;
use lines::linereader::LineReader;

fn main() {
    let (cfg, files) = match Config::from_cmdline(env::args().collect()) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = stderr().write_all(e.as_ref());
            process::exit(1);
        }
    };
    for file in files {
        if let Err(e) = process_arg(&cfg, &file) {
            let _ = write!(&mut stderr(), "{}", e);
            process::exit(1);
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

fn process_arg(cfg: &Config, filename: &str) -> IoResult<()> {
    let f = try!(File::open(filename));

    let mut block = Vec::with_capacity(1024);
    let stdout = stdout();
    let mut stdout = stdout.lock();

    let mut line_no = 0usize;
    let mut collecting_block = false;
    let mut output_produced = false;

    // ~ Collect blocks of lines beloning together and spill them out.
    // ~ A block is defined by two or multiple successive lines of
    // which on the first matches the configured regular expression.
    try_read_lines!(line in LineReader::new(f), {
        line_no += 1;
        if cfg.line_regex.is_match(&String::from_utf8_lossy(line)) {
            if collecting_block {
                // ~ abort as soon as possible if the write didn't
                // succeed e.g. broken pipe
                try!(stdout.write_all(&block));
                output_produced = true;
                collecting_block = false;
            }

            // block.clear();
            unsafe { block.set_len(0); }

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
        let _ = block.write_all(line);
    });
    Ok(())
}
