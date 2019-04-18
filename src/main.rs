#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate regex;
extern crate csv;
extern crate byteorder;
#[macro_use] extern crate structopt;
#[macro_use] extern crate log;
extern crate loggerv;
extern crate glob;

mod types;
mod bit_buffer;
mod encode;
mod decode;
mod template;

use std::fs::File;

use structopt::StructOpt;

use log::{Level};

use encode::*;
use decode::*;
use template::*;


#[derive(Debug, StructOpt)]
#[structopt(name="inlay", about="A command line tool for quickly reading and writing simple binary formats")]
enum Opt {
    #[structopt(name="encode")]
    Encode {
        template_file: String,

        in_files: Vec<String>,

        #[structopt(short="o", long="output", default_value="")]
        out_file: String,

        #[structopt(short="l", long="log-level", default_value="error")]
        log_level: Level,

        #[structopt(short="r", long="rows")]
        rows: bool,
     },

     #[structopt(name="decode")]
     Decode {
        template_file: String,

        in_files: Vec<String>,

        #[structopt(short="o", long="output", default_value="")]
        out_file: String,

        #[structopt(short="l", long="log-level", default_value="error")]
        log_level: Level,

        #[structopt(short="r", long="rows")]
        rows: bool,
     },
}

fn main() {
    let opt = Opt::from_args();

    match opt {
        // Encoding csv into binary
        Opt::Encode { template_file, in_files, out_file, log_level, rows } => {
            loggerv::init_with_level(log_level).unwrap();

            if let Some(templates) = Template::read_templates(&template_file) {
                if in_files.len() > 1 && out_file.len() > 0 {
                    error!("Outfile not supported when run with multiple input files!");
                } else if out_file.len() > 0 {
                    let mut output = File::create(&out_file).expect(&format!("Cannto open output file {}!", out_file));

                    for in_file in in_files {
                        encode(&in_file, &mut output, &templates, rows);
                    }
                } else {
                    for in_file in in_files {
                        let mut out_file = in_file.clone();
                        out_file.push_str(".bin");
                        let mut output =
                            File::create(&out_file).expect(&format!("Cannto open output file {}!", out_file));
                        encode(&in_file, &mut output, &templates, rows);
                    }
                }
            } else {
                panic!("Could not parse template file!");
            }
        },

        // Decoding binary into csv
        Opt::Decode { template_file, in_files, out_file, log_level, rows } => {
            loggerv::init_with_level(log_level).unwrap();

            if in_files.len() > 1 && out_file.len() > 0 {
                error!("Outfile not supported when run with multiple input files!");
            } else {
                if let Some(templates) = Template::read_templates(&template_file) {
                    if out_file.len() > 0 {

                        let mut output_file =
                            File::create(&out_file).expect(&format!("Could not open output file '{}'!", &out_file));

                        for in_file in in_files {
                            decode(&in_file, &mut output_file, &templates, rows);
                        }
                    } else {
                        for in_file in in_files {
                            let mut out_file = in_file.clone();
                            out_file.push_str(".csv");

                            let mut output_file =
                                File::create(&out_file).expect(&format!("Could not open output file '{}'!", &out_file));

                            decode(&in_file, &mut output_file, &templates, rows);
                        }
                    }
                } else {
                    panic!("Could not parse template file!");
                }
            }
        },
    }
}

