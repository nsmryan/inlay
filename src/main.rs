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
use std::io::{Write, Read, Cursor, BufReader};

use structopt::StructOpt;

use log::{Level};

use glob::glob;

use encode::*;
use decode::*;
use template::*;


#[derive(Debug, StructOpt)]
#[structopt(name="inlay", about="A command line tool for quickly reading and writing simple binary formats")]
enum Opt {
    #[structopt(name="encode")]
    Encode {
        template_file: String,

        in_file_globs: Vec<String>,

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

        in_file_globs: Vec<String>,

        #[structopt(short="o", long="output", default_value="")]
        out_file: String,

        #[structopt(short="l", long="log-level", default_value="error")]
        log_level: Level,

        #[structopt(short="r", long="rows")]
        rows: bool,
     },
}

fn expand_globs(input_files: Vec<String>) -> Vec<String> {
    let mut file_names = vec!();
    for file_name in input_files {
        for entry in glob(&file_name).expect(&format!("Could not glob input file name '{}'", file_name)) {
            file_names.push(entry.unwrap().to_str().unwrap().to_string().clone());
        }
    }

    file_names
}

fn main() {
    let opt = Opt::from_args();

    match opt {
        // Encoding csv into binary
        Opt::Encode { template_file, in_file_globs, out_file, log_level, rows } => {
            loggerv::init_with_level(log_level).unwrap();
             trace!("Encoding");

            trace!("Expanding globs");
            let mut in_files = expand_globs(in_file_globs);

            let input_files_given = in_files.len();

            // if no input files are given, and in row mode,
            // use the template file itself
            // column mode has a different syntax, so the template file would be invalid
            if rows && in_files.len() == 0 {
                trace!("Template file is input file");
                in_files.push(template_file.clone());
            }

            if in_files.len() == 0 {
                panic!("No input files to be processed!");
            }

            // open template file
            if let Some(templates) = Template::read_templates(&template_file) {
                trace!("Template file open");
                // if input files were provided, and an output file was given
                if input_files_given > 1 && out_file.len() > 0 {
                    error!("Outfile not supported when run with multiple input files!");
                } else if out_file.len() > 0 { // otherwise, if an output file was given
                    trace!("Single output file {}", out_file);
                    let mut output = File::create(&out_file).expect(&format!("Cannot open output file {}!", out_file));
                    trace!("Output file open");

                    trace!("{} input files to process", in_files.len());
                    for in_file in in_files {
                        trace!("Processing input file {}", in_file);

                        let mut input = File::open(&in_file)
                                              .or_else(|err| { error!("Could not open input file '{}'!", &in_file);
                                                                        Err(err)
                                                                       }).ok().unwrap();
                        if let None = encode(&mut input, &mut output, &templates, rows) {
                            panic!("Encoding error!");
                        } else {
                            trace!("File processed");
                        }
                    }
                } else { // otherwise create output file name from input file names
                    trace!("Multiple output files");

                    trace!("{} input files to process", in_files.len());
                    for in_file in in_files {
                        let mut out_file = in_file.clone();
                        out_file.push_str(".bin");
                        info!("Outputting to {}", out_file);

                        info!("Processing input file {}", in_file);

                        let mut output =
                            File::create(&out_file).expect(&format!("Cannot open output file {}!", out_file));

                        let mut input = File::open(&in_file).or_else(|err| { error!("Could not open input file '{}'!", &in_file);
                                                                        Err(err)
                                                                       }).ok().unwrap();

                        if let None = encode(&mut input, &mut output, &templates, rows) {
                            panic!("Encoding error!");
                        } else {
                            trace!("File processed");
                        }
                    }
                }
            } else {
                panic!("Could not parse template file!");
            }
        },

        // Decoding binary into csv
        Opt::Decode { template_file, in_file_globs, out_file, log_level, rows } => {
            loggerv::init_with_level(log_level).unwrap();

             trace!("Decoding");

             trace!("Expanding globs");
            let in_files = expand_globs(in_file_globs);

            if in_files.len() == 0 {
                panic!("No input files to be processed!");
            }

            if in_files.len() > 1 && out_file.len() > 0 {
                error!("Outfile not supported when run with multiple input files!");
            } else {
                 trace!("Opening template file");
                // open template file
                if let Some(templates) = Template::read_templates(&template_file) {
                     trace!("Template file opened");

                    // if an output file was provided, write all output to that file.
                    if out_file.len() > 0 {
                        trace!("Single output file");
                        info!("Outputting to {}", out_file);

                        let mut output_file =
                            File::create(&out_file).expect(&format!("Could not open output file '{}'!", &out_file));

                        trace!("{} input files to process", in_files.len());
                        for in_file in in_files {
                            info!("Processing input file {}", in_file);
                            let input_file =
                                File::open(&in_file).expect(&format!("Could not open input file '{}'!", &in_file));
                            let mut input = BufReader::new(input_file);

                            if let None = decode(&mut input, &mut output_file, &templates, rows) {
                                panic!("Error decoding!");
                            } else {
                                trace!("File processed");
                            }
                        }
                    } else { // otherwise construct an output file for each input file.
                        trace!("Multiple output files");

                        trace!("{} input files to process", in_files.len());
                        for in_file in in_files {
                            trace!("Processing input file {}", in_file);

                            let mut out_file = in_file.clone();
                            out_file.push_str(".csv");
                            trace!("Outputting to {}", out_file);

                            let mut output_file =
                                File::create(&out_file).expect(&format!("Could not open output file '{}'!", &out_file));
                            trace!("Output file open");

                            let input_file =
                                File::open(&in_file).expect(&format!("Could not open input file '{}'!", &in_file));
                            let mut input = BufReader::new(input_file);
                            trace!("Input file open");

                            if let None = decode(&mut input, &mut output_file, &templates, rows) {
                                panic!("Error decoding!");
                            } else {
                                trace!("File processed");
                            }
                        }
                    }
                } else {
                    panic!("Could not parse template file!");
                }
            }
        },
    }
}

