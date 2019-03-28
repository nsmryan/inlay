#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate regex;
extern crate csv;
extern crate byteorder;
#[macro_use] extern crate structopt;
#[macro_use]
extern crate log;
extern crate loggerv;
extern crate glob;


use std::io::{Read, Write};
use std::io::BufReader;
use std::fs::File;

use byteorder::{LittleEndian, BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};

use structopt::StructOpt;

use loggerv::*;
use log::{Level};

use regex::Regex;

use glob::glob;

mod types;
use types::*;


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


fn to_field(typ: FieldType, value_str: &str, description: String) -> Field {
    let value = to_value(typ, value_str);
    Field {
        value: value,
        typ: typ,
        description: description,
    }
}

fn to_value(typ: FieldType, value_str: &str) -> Value {
  dbg!(value_str);
  match typ {
    FieldType::Int(num_bits, _) => {
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

    FieldType::Uint(num_bits, _) => {
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

    FieldType::Float(_) => {
        Value::Float(value_str.parse().ok().unwrap())
    },

    FieldType::Double(_) => {
        Value::Double(value_str.parse().ok().unwrap())
    },
  }
}

fn write_out<R>(reader: &mut R, field: &Field, bit_buffer: &mut BitBuffer)
    where R: Read + WriteBytesExt {

    // the common case is that we are writing fields that fit into full bytes.
    if bit_buffer.bits_avail == 0 && field.full_width() {
        match field.typ.endianness() {
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
    // otherwise, do bit level writes
    } else {
        //println!("before bit_buffer.bits = {:b}, {} bits", bit_buffer.bits, field.typ.num_bits());
        bit_buffer.push_value(field.value, field.typ.num_bits(), field.typ.endianness());
        //println!("after  bit_buffer.bits = {:b}", bit_buffer.bits);

        //println!("bits avail = {}", bit_buffer.bits_avail);
        if bit_buffer.bits_avail % 8 == 0 {
            // println!("Writing out {} bytes", bit_buffer.bits_avail / 8);
            for _ in 0..(bit_buffer.bits_avail / 8) {
                reader.write(&[bit_buffer.pull_byte()]);
            }
        }
    }
}

fn read_field<R>(reader: &mut R,
                 bit_buffer: &mut BitBuffer,
                 template: &Template) -> Option<Field>
    where R: ReadBytesExt {

    let num_bits = template.typ.num_bits();

    match template.typ {
        FieldType::Int(_, _) | FieldType::Uint(_, _) => {
            let value: Value;

            // while we need more bits, push bytes from the reader into the decoder
            while (bit_buffer.bits_avail as usize) < num_bits {
                let byte = reader.read_u8().ok()?;
                dbg!(byte);

                match template.typ.endianness() {
                    Endianness::Little => bit_buffer.push_byte_le(byte)?,
                    Endianness::Big    => bit_buffer.push_byte_be(byte)?,
                }
            }

            match template.typ {
                FieldType::Int(_,  _) => { value = bit_buffer.pull_value_int(num_bits as u8)?; },
                FieldType::Uint(_, _) => { value = bit_buffer.pull_value_uint(num_bits as u8)?; },
                _ => panic!("This case should have been guarded by an above match!"),
            }

            Some(Field {
                value: value,
                typ: template.typ,
                description: template.description.clone(),
            })
        },

        FieldType::Float(endianness) => {
            if bit_buffer.bits_avail != 0 {
                error!("Tried to read a float from a bit offset!");
                None
            } else {
                let value;

                if endianness == Endianness::Little {
                    value = Value::Float(reader.read_f32::<LittleEndian>().ok()?);
                } else {
                    value = Value::Float(reader.read_f32::<BigEndian>().ok()?);
                }

                Some(Field {
                    value: value,
                    typ: template.typ,
                    description: template.description.clone(),
                })
            }
        },

        FieldType::Double(endianness) => {
            if bit_buffer.bits_avail != 0 {
                error!("Tried to read a double from a bit offset!");
                None
            } else {
                let value;
                if endianness == Endianness::Little {
                    value = Value::Double(reader.read_f64::<LittleEndian>().ok()?);
                } else {
                    value = Value::Double(reader.read_f64::<BigEndian>().ok()?);
                }

                Some(Field {
                    value: value,
                    typ: template.typ,
                    description: template.description.clone(),
                })
            }
        },
    }
}

fn write_field<W: Write>(writer: &mut W, field: &Field, description: &String) {
    writer.write_all(&field.to_record().as_bytes());
}

// NOTE It would be better to use a syntax like
// uint8_be:3 so you can specify both a type and a bitwidth, with the bitwidth
// optional
fn parse_type(type_str: &str) -> Option<FieldType> {
    let type_str = type_str.to_lowercase();

    lazy_static! {
      static ref TYPE_REGEX: Regex =
          Regex::new(r"(float|double|int|uint)(\d{0,2})_(be|le)").unwrap();
    }
    let matches = TYPE_REGEX.captures(&type_str)?;

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
            error!("Type '{}' unexpected in field type '{}'", &matches[1], type_str);
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

    let mut bit_buffer: BitBuffer = Default::default();

    for record in lines.records() {
        let mut rec = record.ok()?;

        let type_str = &rec[0];
        let description = &rec[1];
        let value_str = &rec[2];

        let typ = parse_type(type_str)?;

        let field = to_field(typ, value_str, description.to_string());
        info!("{}", field);

        write_out(&mut output, &field, &mut bit_buffer);
    }

    info!("Finished writing to {}", &out_file);

    Some(())
}

fn decode(in_file: &String, out_file: &String, template_file: &String, repetitions: isize) -> Option<()> {
    let mut input_file =
        File::open(&in_file).expect(&format!("Could not open input file '{}'!", &in_file));
    let mut input = BufReader::new(input_file);

    let template: File =
        File::open(&template_file).expect(&format!("Could not open template file '{}'!", &template_file));
    let mut lines = csv::Reader::from_reader(&template);
    info!("Opened {}", &template_file);

    let header_line = lines.headers().ok()?;

    let mut output_file =
        File::create(&out_file).expect(&format!("Could not open output file '{}'!", &out_file));


    output_file.write_all(&"type,description,value\n".to_string().as_bytes());
    let mut templates: Vec<Template> = vec!();
    for record in lines.records() {
        let mut rec = record.ok()?;
        let typ = parse_type(&rec[0])?;
        let desc = rec[1].to_string();

        let template: Template =
            Template {
                typ: typ,
                description: desc,
            };

        templates.push(template);
    }

    let mut decoder_state = Default::default();
    for index in 0..repetitions {
        for template in templates.iter() {
            let field = read_field(&mut input, &mut decoder_state, &template)?;
            info!("{}", field);

            write_field(&mut output_file, &field, &template.description);

            output_file.write_all(&b"\n"[..]).unwrap();
        }
    }

    info!("Finished writing to {}", &out_file);

    Some(())
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

