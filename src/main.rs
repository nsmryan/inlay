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
     },

     #[structopt(name="decode")]
     Decode {
        in_file: String,

        #[structopt(short="o", long="output", default_value="data.csv")]
        out_file: String,

        #[structopt(short="t", long="template")]
        template_file: String,

        #[structopt(short="r", long="repeat", default_value="1")]
        repetitions: usize,
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

fn read_field<R>(reader: &mut R, template: &Template) -> Field
    where R: ReadBytesExt {
    match template.typ {
        FieldType::Int(num_bits, endianness) => {
            let value;
            if endianness == Endianness::Little {
                if num_bits <= 8 {
                    value = Value::Uint8(reader.read_u8().unwrap());
                } else if num_bits <= 16 {
                    value = Value::Uint16(reader.read_u16::<LittleEndian>().unwrap());
                } else if num_bits <= 32 {
                    value = Value::Uint32(reader.read_u32::<LittleEndian>().unwrap());
                } else if num_bits <= 64 {
                    value = Value::Uint64(reader.read_u64::<LittleEndian>().unwrap());
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            } else {
                if num_bits <= 8 {
                    value = Value::Uint8(reader.read_u8().unwrap());
                } else if num_bits <= 16 {
                    value = Value::Uint16(reader.read_u16::<BigEndian>().unwrap());
                } else if num_bits <= 32 {
                    value = Value::Uint32(reader.read_u32::<BigEndian>().unwrap());
                } else if num_bits <= 64 {
                    value = Value::Uint64(reader.read_u64::<BigEndian>().unwrap());
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            }

            Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            }
        },

        FieldType::Uint(num_bits, endianness) => {
            let value;
            if endianness == Endianness::Little {
                if num_bits <= 8 {
                    value = Value::Int8(reader.read_i8().unwrap());
                } else if num_bits <= 16 {
                    value = Value::Int16(reader.read_i16::<LittleEndian>().unwrap());
                } else if num_bits <= 32 {
                    value = Value::Int32(reader.read_i32::<LittleEndian>().unwrap());
                } else if num_bits <= 64 {
                    value = Value::Int64(reader.read_i64::<LittleEndian>().unwrap());
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            } else {
                if num_bits <= 8 {
                    value = Value::Int8(reader.read_i8().unwrap());
                } else if num_bits <= 16 {
                    value = Value::Int16(reader.read_i16::<BigEndian>().unwrap());
                } else if num_bits <= 32 {
                    value = Value::Int32(reader.read_i32::<BigEndian>().unwrap());
                } else if num_bits <= 64 {
                    value = Value::Int64(reader.read_i64::<BigEndian>().unwrap());
                } else {
                    panic!("{} bits in a field are not supported!");
                }
            }

            Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            }
        },

        FieldType::Float(endianness) => {
            let value;
            if endianness == Endianness::Little {
                value = Value::Float(reader.read_f32::<LittleEndian>().unwrap());
            } else {
                value = Value::Float(reader.read_f32::<BigEndian>().unwrap());
            }

            Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            }
        },

        FieldType::Double(endianness) => {
            let value;
            if endianness == Endianness::Little {
                value = Value::Double(reader.read_f64::<LittleEndian>().unwrap());
            } else {
                value = Value::Double(reader.read_f64::<BigEndian>().unwrap());
            }

            Field {
                value: value,
                endianness: endianness,
                typ: template.typ,
                description: template.description.clone(),
            }
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
    let matches = TYPE_REGEX.captures(&typ_str).unwrap();

    match &matches[1] {
        "uint" => {
            let num_bits = matches[2].parse::<NumBits>().unwrap();

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
            let num_bits = matches[2].parse::<NumBits>().unwrap();

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

fn encode(in_file: &String, out_file: &String) {
    let file;

    match File::open(&in_file) {
        Ok(file_handle) => file = file_handle,
        Err(_) => {
            error!("Could not open input file '{}'!", &in_file);
            return;
        },
    }

    info!("Opened {}", &in_file);

    let mut lines = csv::Reader::from_reader(file);

    let header_line = lines.headers().unwrap();

    let mut output = File::create(&out_file).unwrap();

    let mut endianness = Endianness::Big;

    for record in lines.records() {
        let mut rec = record.unwrap();

        let typ_str = &rec[0];
        let description = &rec[1];
        let value_str = &rec[2];

        let typ = parse_typ(typ_str).unwrap();

        let field = to_field(typ, value_str, description.to_string());
        info!("{}", field);

        write_out(&mut output, &field, &field.endianness);
    }

    info!("Finished writing to {}", &out_file);
}

fn decode(in_file: &String, out_file: &String, template_file: &String, repetitions: usize) {
    let mut input_file;

    match File::open(&in_file) {
        Ok(file_handle) => input_file = file_handle,
        Err(_) => {
            error!("Could not open input file '{}'!", &in_file);
            return;
        },
    }
    let mut input = BufReader::new(input_file);

    let template;
    match File::open(&template_file) {
        Ok(file_handle) => template = file_handle,
        Err(_) => {
            error!("Could not open template file '{}'!", &template_file);
            return;
        },
    }

    info!("Opened {}", &template_file);

    let mut lines = csv::Reader::from_reader(&template);

    let header_line = lines.headers().unwrap();

    // NOTE ensure good error messages here
    let mut output;
    match File::create(&out_file) {
        Ok(file_handle) => output = file_handle,
        Err(_) => {
            error!("Could not open output file '{}'!", &out_file);
            return;
        },
    }

    output.write_all(&"typ,description,value\n".to_string().as_bytes());
    // NOTE parse manually to provide better error messages
    let mut templates: Vec<Template> = vec!();
    for record in lines.records() {
        let mut rec = record.unwrap();

        let template: Template =
            Template {
                typ: parse_typ(&rec[0]).unwrap(),
                description: rec[1].to_string(),
            };

        templates.push(template);
    }

    for _ in 0..repetitions {
        for template in templates.iter() {
            let field = read_field(&mut input, &template);
            info!("{}", field);

            write_field(&mut output, &field, &template.description);

            output.write_all(&b"\n"[..]);
        }
    }

    info!("Finished writing to {}", &out_file);
}

fn main() {
    let opt = Opt::from_args();

    loggerv::init_with_level(Level::Info).unwrap();

    match opt {
        Opt::Encode { in_file, out_file } => {
            encode(&in_file, &out_file);
        },

        Opt::Decode { in_file, out_file, template_file, repetitions } => {
            decode(&in_file, &out_file, &template_file, repetitions);
        },
    }

}

