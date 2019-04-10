#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate regex;
extern crate csv;
extern crate byteorder;
#[macro_use] extern crate structopt;
#[macro_use] extern crate log;
extern crate loggerv;
extern crate glob;


use std::fs::File;

use structopt::StructOpt;

use log::{Level};

mod types;

mod bit_buffer;

mod encode;
use encode::*;

mod decode;
use decode::*;


#[derive(Debug, StructOpt)]
#[structopt(name="inlay", about="A command line tool for quickly reading and writing simple binary formats")]
enum Opt {
    #[structopt(name="encode")]
    Encode {
        in_files: Vec<String>,

        #[structopt(short="o", long="output", default_value="")]
        out_file: String,

        #[structopt(short="l", long="log-level", default_value="info")]
        log_level: Level,
     },

     #[structopt(name="decode")]
     Decode {
        in_files: Vec<String>,

        #[structopt(short="o", long="output", default_value="")]
        out_file: String,

        #[structopt(short="t", long="template")]
        template_file: String,

        #[structopt(short="r", long="repeat", default_value="1")]
        repetitions: isize,

        #[structopt(short="l", long="log-level", default_value="info")]
        log_level: Level,
     },
}

fn main() {
    let opt = Opt::from_args();


    match opt {
        Opt::Encode { in_files, out_file, log_level} => {
            loggerv::init_with_level(log_level).unwrap();

            if in_files.len() > 1 && out_file.len() > 0 {
                error!("Outfile not supported when run with multiple input files!");
            } else if out_file.len() > 0 {
                for in_file in in_files {
                    encode(&in_file, &out_file);
                }
            } else {
                for in_file in in_files {
                    let mut out_file = in_file.clone();
                    out_file.push_str(".bin");
                    encode(&in_file, &out_file);
                }
            }
        },

        Opt::Decode { in_files, out_file, template_file, repetitions, log_level } => {
            loggerv::init_with_level(log_level).unwrap();

            if in_files.len() > 1 && out_file.len() > 0 {
                error!("Outfile not supported when run with multiple input files!");
            } else if out_file.len() > 0 {
                for in_file in in_files {
                    decode(&in_file, &out_file, &template_file, repetitions);
                }
            } else {
                for in_file in in_files {
                    let mut out_file = in_file.clone();
                    out_file.push_str(".csv");
                    decode(&in_file, &out_file, &template_file, repetitions);
                }
            }
        },
    }
}

