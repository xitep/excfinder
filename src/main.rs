#![feature(if_let)]
#![feature(phase)]

#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate lines;

use std::os;
use std::io::{File, IoResult, BufferedReader};
use std::io::stdio::{stderr, stdout};

use lines::linemapper;

use buf::MemWriter;

mod buf;

fn main() {
    let args = os::args();
    if args.len() < 2 {
        let _ = write!(stderr(), "usage: {:s} <logfile> [<logfile> ...]\n", args[0]);
        std::os::set_exit_status(1);
        return;
    }
    for arg in args.iter().skip(1) {
        if let Err(e) = process_arg(arg.as_slice()) {
            let _ = write!(stderr(), "{}", e);
            std::os::set_exit_status(1);
            return;
        }
    }
}

#[allow(unused_must_use)]
fn process_arg(arg: &str) -> IoResult<()> {
    let f = try!(File::open(&Path::new(arg)));
    let r = BufferedReader::new(f);

    let entry_start = regex!(r"^\d{4}-");

    let mut block = MemWriter::with_capacity(1024);
    let mut stdout = stdout().unwrap();

    let mut line_no = 0u;
    let mut collecting_block = false;
    try!(linemapper::map_lines(r, |line| {
        line_no += 1;
        if entry_start.is_match(String::from_utf8_lossy(line).as_slice()) {
            if collecting_block {
                block.write_char('\n');
                // ~ abort as soon as possible if the write didn't
                // succeed e.g. broken pipe
                if let Err(e) = stdout.write(block.get_ref()) {
                    write!(stderr(), "{}", e);
                    std::os::set_exit_status(1);
                    return false;
                }
                collecting_block = false;
            }
            block.clear();
        } else {
            collecting_block = true;
        }
        write!(block, "{:7u}  ", line_no);
        block.write(line);
        true
    }));
    
    Ok(())
}
