///
/// Inlay
///
/// This program provides a very simple translation from binary data to csv, and csv
/// to binary data. It is intended for working with trivial binary formats, and especially
/// for initial testing and development while there may not be tools specific to the format
/// to use.
///
/// CSV -> Binary
/// When building a binary file, a csv file in the following format must be provided:
/// type,description,value
/// uint8_be,field1,1
/// uint16_be,field2,141
/// uint32_be,field3,1245
///
///
/// The header must have fields typ, description, and value, in that order. The types
/// are given by the following regex:
/// (uint8|uint16|uint32|uint64|int8|int16|int32|int64|float|double)_(be|le)
///
/// Any field can be big endian or little endian, allowing mixed endianness within
/// a file.
///
/// Binary -> CSV
/// When translating binary to CSV, a file of the following format must be provided:
/// type,description
/// uint64_be,a 64 bit field
/// uint8_be,an 8 bit field
///
/// This file must have a header "type,description" and any number of entries with
/// a type as defined above. The description is optional and will be copied into the
/// output csv file.
///
/// The output csv file is in exactly the same format as the input csv file when 
/// encoding from CSV -> Binary. This means that a binary structure can be decoded,
/// modified and written back.
/// To assist with this use case, if the "template" file given during decoding has
/// a "values" column it will be ignored. This allows a csv file from decoding to be used as the
/// template when decoding other instances of a binary structure.
///
///
/// When decoding, a file many have many entries. The command line options allow repetitions
/// of a certain number of repeated structures, or as many as necessary to read the whole file.
/// Structopt seems to require the flag "-r=-1" rather then "-r -1" when specifying negative
/// numbers.
///
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate regex;
extern crate csv;
extern crate byteorder;
#[macro_use] extern crate structopt;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate bitstream_io;


use std::io::{Read, Write};
use std::io::BufReader;
use std::fs::File;

use byteorder::{LittleEndian, BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};

use structopt::StructOpt;

use loggerv::*;
use log::{Level};

use regex::Regex;

mod types;
use types::*;


#[derive(Debug, StructOpt)]
#[structopt(name="inlay", about="A command line tool for quickly reading and writing simple binary formats")]
enum Opt {
    #[structopt(name="encode")]
    Encode {
        in_file: String,

        #[structopt(short="o", long="output", default_value="data.bin")]
        out_file: String,

        #[structopt(short="l", long="log-level", default_value="Level::info")]
        log_level: Level,
     },

     #[structopt(name="decode")]
     Decode {
        in_file: String,

        #[structopt(short="o", long="output", default_value="data.csv")]
        out_file: String,

        #[structopt(short="t", long="template")]
        template_file: String,

        #[structopt(short="r", long="repeat", default_value="1")]
        repetitions: isize,

        #[structopt(short="l", long="log-level", default_value="Level::info")]
        log_level: Level,
     },
}


fn to_field(typ: FieldType, value_str: &str, description: String) -> Field {
    let value = to_value(typ, value_str);
    Field {
        value: value,
        endianness: typ.endianness(),
        typ: typ,
        description: description,
    }
}

fn to_value(typ: FieldType, value_str: &str) -> Value {
  match typ {
    FieldType::Int(num_bits, _) => {
        if num_bits <= 8 {
            Value::Uint8(value_str.parse().ok().unwrap())
        } else if num_bits <= 16 {
            Value::Uint16(value_str.parse().ok().unwrap())
        } else if num_bits <= 32 {
            Value::Uint32(value_str.parse().ok().unwrap())
        } else if num_bits <= 64 {
            Value::Uint64(value_str.parse().ok().unwrap())
        } else {
            panic!("{} bit fields are not allowed!", num_bits);
        }
    },

    FieldType::Uint(num_bits, _) => {
        if num_bits <= 8 {
            Value::Int8(value_str.parse().ok().unwrap())
        } else if num_bits <= 16 {
            Value::Int16(value_str.parse().ok().unwrap())
        } else if num_bits <= 32 {
            Value::Int32(value_str.parse().ok().unwrap())
        } else if num_bits <= 64 {
            Value::Int64(value_str.parse().ok().unwrap())
        } else {
            panic!("{} bit fields are not allowed!", num_bits);
        }
    },

    FieldType::Float(_) => {
        Value::Float(value_str.parse().ok().unwrap())
    },

    FieldType::Double(_) => {
        Value::Double(value_str.parse().ok().unwrap())
    },
  }
}

fn write_out<R>(reader: &mut R, field: &Field, endianness: &Endianness)
    where R: Read + WriteBytesExt {

    match field.endianness {
        Endianness::Big => {
          match field.value {
            Value::Uint8(val) => { reader.write(&[val]).ok(); },
            Value::Int8(val) => { reader.write(&[val as u8]).ok(); },
            Value::Uint16(val) => { reader.write_u16::<BigEndian>(val).ok(); },
            Value::Int16(val) => { reader.write_i16::<BigEndian>(val).ok(); },
            Value::Uint32(val) => { reader.write_u32::<BigEndian>(val).ok(); },
            Value::Int32(val) => { reader.write_i32::<BigEndian>(val).ok(); },
            Value::Uint64(val) => { reader.write_u64::<BigEndian>(val).ok(); },
            Value::Int64(val) => { reader.write_i64::<BigEndian>(val).ok(); },
            Value::Float(val) => { reader.write_f32::<BigEndian>(val).ok(); },
            Value::Double(val) => { reader.write_f64::<BigEndian>(val).ok(); },
          }
        },

        Endianness::Little => {
          match field.value {
            Value::Uint8(val) => { reader.write(&[val]).ok(); },
            Value::Int8(val) => { reader.write(&[val as u8]).ok(); },
            Value::Uint16(val) => { reader.write_u16::<LittleEndian>(val).ok(); },
            Value::Int16(val) => { reader.write_i16::<LittleEndian>(val).ok(); },
            Value::Uint32(val) => { reader.write_u32::<LittleEndian>(val).ok(); },
            Value::Int32(val) => { reader.write_i32::<LittleEndian>(val).ok(); },
            Value::Uint64(val) => { reader.write_u64::<LittleEndian>(val).ok(); },
            Value::Int64(val) => { reader.write_i64::<LittleEndian>(val).ok(); },
            Value::Float(val) => { reader.write_f32::<LittleEndian>(val).ok(); },
            Value::Double(val) => { reader.write_f64::<LittleEndian>(val).ok(); },
          }
        },
    }
}

fn read_field<R>(reader: &mut R, template: &Template) -> Option<Field>
    where R: ReadBytesExt {
    match template.typ {
        FieldType::Int(num_bits, endianness) => {
            let value;
            if endianness == Endianness::Little {
                if num_bits <= 8 {
                    value = Value::Uint8(reader.read_u8().ok()?);
                } else if num_bits <= 16 {
                    value = Value::Uint16(reader.read_u16::<LittleEndian>().ok()?);
                } else if num_bits <= 32 {
                    value = Value::Uint32(reader.read_u32::<LittleEndian>().ok()?);
                } else if num_bits <= 64 {
                    value = Value::Uint64(reader.read_u64::<LittleEndian>().unwrap());
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            } else {
                if num_bits <= 8 {
                    value = Value::Uint8(reader.read_u8().ok()?);
                } else if num_bits <= 16 {
                    value = Value::Uint16(reader.read_u16::<BigEndian>().ok()?);
                } else if num_bits <= 32 {
                    value = Value::Uint32(reader.read_u32::<BigEndian>().ok()?);
                } else if num_bits <= 64 {
                    value = Value::Uint64(reader.read_u64::<BigEndian>().ok()?);
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            }

            Some(Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            })
        },

        FieldType::Uint(num_bits, endianness) => {
            let value;
            if endianness == Endianness::Little {
                if num_bits <= 8 {
                    value = Value::Int8(reader.read_i8().ok()?);
                } else if num_bits <= 16 {
                    value = Value::Int16(reader.read_i16::<LittleEndian>().ok()?);
                } else if num_bits <= 32 {
                    value = Value::Int32(reader.read_i32::<LittleEndian>().ok()?);
                } else if num_bits <= 64 {
                    value = Value::Int64(reader.read_i64::<LittleEndian>().ok()?);
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            } else {
                if num_bits <= 8 {
                    value = Value::Int8(reader.read_i8().ok()?);
                } else if num_bits <= 16 {
                    value = Value::Int16(reader.read_i16::<BigEndian>().ok()?);
                } else if num_bits <= 32 {
                    value = Value::Int32(reader.read_i32::<BigEndian>().ok()?);
                } else if num_bits <= 64 {
                    value = Value::Int64(reader.read_i64::<BigEndian>().ok()?);
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            }

            Some(Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            })
        },

        FieldType::Float(endianness) => {
            let value;
            if endianness == Endianness::Little {
                value = Value::Float(reader.read_f32::<LittleEndian>().ok()?);
            } else {
                value = Value::Float(reader.read_f32::<BigEndian>().ok()?);
            }

            Some(Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            })
        },

        FieldType::Double(endianness) => {
            let value;
            if endianness == Endianness::Little {
                value = Value::Double(reader.read_f64::<LittleEndian>().ok()?);
            } else {
                value = Value::Double(reader.read_f64::<BigEndian>().ok()?);
            }

            Some(Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            })
        },
    }
}

fn write_field<W: Write>(writer: &mut W, field: &Field, description: &String) {
    writer.write_all(&field.to_record().as_bytes());
}

fn parse_typ(typ_str: &str) -> Option<FieldType> {
    let typ_str = typ_str.to_lowercase();

    lazy_static! {
      static ref TYPE_REGEX: Regex =
          Regex::new(r"(float|double|int|uint)(\d{0,2})_(be|le)").unwrap();
    }
    let matches = TYPE_REGEX.captures(&typ_str)?;

    match &matches[1] {
        "uint" => {
            let num_bits = matches[2].parse::<NumBits>().ok()?;

            match &matches[3] {
                "be" => Some(FieldType::Uint(num_bits, Endianness::Big)),

                "le" => Some(FieldType::Uint(num_bits, Endianness::Little)),

                 _ => {
                     error!("Endianness '{}' not expected!", &matches[3]);
                     None
                 },
            }
        },

        "int" => {
            let num_bits = matches[2].parse::<NumBits>().ok()?;

            match &matches[3] {
                "be" => Some(FieldType::Int(num_bits, Endianness::Big)),

                "le" => Some(FieldType::Int(num_bits, Endianness::Little)),

                 _ => {
                     error!("Endianness '{}' not expected!", &matches[3]);
                     None
                 },
            }
        },

        "float" => {
            // NOTE should check that no bit size is given
            match &matches[3] {
                "be" => Some(FieldType::Float(Endianness::Big)),

                "le" => Some(FieldType::Float(Endianness::Little)),

                 _ => {
                     error!("Endianness '{}' not expected!", &matches[3]);
                     None
                 },
            }
        },

        "double" => {
            // NOTE should check that no bit size is given
            match &matches[3] {
                "be" => Some(FieldType::Double(Endianness::Big)),

                "le" => Some(FieldType::Double(Endianness::Little)),

                 _ => {
                     error!("Endianness '{}' not expected!", &matches[3]);
                     None
                 },
            }
        },

        _ => {
            dbg!(&matches);
            error!("Type '{}' unexpected in field type '{}'", &matches[1], typ_str);
            None
        }
    }
}

fn encode(in_file: &String, out_file: &String) -> Option<()> {
    let file;

    match File::open(&in_file) {
        Ok(file_handle) => file = file_handle,
        Err(_) => {
            error!("Could not open input file '{}'!", &in_file);
            return Some(());
        },
    }

    info!("Opened {}", &in_file);

    let mut lines = csv::Reader::from_reader(file);

    let header_line = lines.headers().ok()?;

    let mut output = File::create(&out_file).ok()?;

    let mut endianness = Endianness::Big;

    for record in lines.records() {
        let mut rec = record.ok()?;

        let typ_str = &rec[0];
        let description = &rec[1];
        let value_str = &rec[2];

        let typ = parse_typ(typ_str)?;

        let field = to_field(typ, value_str, description.to_string());
        info!("{}", field);

        write_out(&mut output, &field, &field.endianness);
    }

    info!("Finished writing to {}", &out_file);

    Some(())
}

fn decode(in_file: &String, out_file: &String, template_file: &String, repetitions: isize) -> Option<()> {
    let mut input_file;

    match File::open(&in_file) {
        Ok(file_handle) => input_file = file_handle,
        Err(_) => {
            error!("Could not open input file '{}'!", &in_file);
            return Some(());
        },
    }
    let mut input = BufReader::new(input_file);

    let template;
    match File::open(&template_file) {
        Ok(file_handle) => template = file_handle,
        Err(_) => {
            error!("Could not open template file '{}'!", &template_file);
            return Some(());
        },
    }

    info!("Opened {}", &template_file);

    let mut lines = csv::Reader::from_reader(&template);

    let header_line = lines.headers().ok()?;

    // NOTE ensure good error messages here
    let mut output;
    match File::create(&out_file) {
        Ok(file_handle) => output = file_handle,
        Err(_) => {
            error!("Could not open output file '{}'!", &out_file);
            return Some(());
        },
    }

    output.write_all(&"type,description,value\n".to_string().as_bytes());
    // NOTE parse manually to provide better error messages
    let mut templates: Vec<Template> = vec!();
    for record in lines.records() {
        let mut rec = record.ok()?;

        let template: Template =
            Template {
                typ: parse_typ(&rec[0])?,
                description: rec[1].to_string(),
            };

        templates.push(template);
    }

    let mut index = repetitions;
    // negative numbers will repeat to end of file
    while index != 0 {
        for template in templates.iter() {
            let field = read_field(&mut input, &template)?;
            info!("{}", field);

            write_field(&mut output, &field, &template.description);

            output.write_all(&b"\n"[..]);
        }

        index -= 1;
    }

    info!("Finished writing to {}", &out_file);

    Some(())
}

fn main() {
    let opt = Opt::from_args();


    match opt {
        Opt::Encode { in_file, out_file, log_level} => {
            loggerv::init_with_level(log_level).unwrap();
            encode(&in_file, &out_file);
        },

        Opt::Decode { in_file, out_file, template_file, repetitions, log_level } => {
            loggerv::init_with_level(log_level).unwrap();
            decode(&in_file, &out_file, &template_file, repetitions);
        },
    }

}

