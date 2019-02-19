#[macro_use] extern crate serde_derive;
extern crate csv;
extern crate byteorder;
#[macro_use] extern crate structopt;
#[macro_use]
extern crate log;
extern crate loggerv;


use std::io::{Read, Write};
use std::fs::File;

use byteorder::{LittleEndian, BigEndian, ByteOrder, WriteBytesExt};

use structopt::StructOpt;

use loggerv::*;
use log::{Level};

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
     },
}


fn to_field(typ: FieldType, value_str: String) -> Field {
    let value = to_value(typ, value_str);
    Field {
        value: value,
        endianness: typ.endianness()
    }
}

fn to_value(typ: FieldType, value_str: String) -> Value {
  match typ {
    FieldType::uint8_be | FieldType::uint8_le =>
        Value::Uint8(value_str.parse().ok().unwrap()),
    FieldType::int8_be | FieldType::int8_le =>
        Value::Int8(value_str.parse().ok().unwrap()),
    FieldType::uint16_be | FieldType::uint16_le =>
        Value::Uint16(value_str.parse().ok().unwrap()),
    FieldType::int16_be | FieldType::int16_le =>
        Value::Int16(value_str.parse().ok().unwrap()),
    FieldType::uint32_be | FieldType::uint32_le =>
        Value::Uint32(value_str.parse().ok().unwrap()),
    FieldType::int32_be | FieldType::int32_le =>
        Value::Int32(value_str.parse().ok().unwrap()),
    FieldType::float_be | FieldType::float_le =>
        Value::Float(value_str.parse().ok().unwrap()),
    FieldType::double_be | FieldType::double_le =>
        Value::Double(value_str.parse().ok().unwrap()),
  }
}

fn write_out<O: Write>(output: &mut O, field: Field) {
    match field.endianness {
        Endianness::Big => {
          match field.value {
            Value::Uint8(val) => { output.write(&[val]).ok(); },
            Value::Int8(val) => { output.write(&[val as u8]).ok(); },
            Value::Uint16(val) => { output.write_u16::<BigEndian>(val).ok(); },
            Value::Int16(val) => { output.write_i16::<BigEndian>(val).ok(); },
            Value::Uint32(val) => { output.write_u32::<BigEndian>(val).ok(); },
            Value::Int32(val) => { output.write_i32::<BigEndian>(val).ok(); },
            Value::Float(val) => { output.write_f32::<BigEndian>(val).ok(); },
            Value::Double(val) => { output.write_f64::<BigEndian>(val).ok(); },
          }
        },

        Endianness::Little => {
          match field.value {
            Value::Uint8(val) => { output.write(&[val]).ok(); },
            Value::Int8(val) => { output.write(&[val as u8]).ok(); },
            Value::Uint16(val) => { output.write_u16::<LittleEndian>(val).ok(); },
            Value::Int16(val) => { output.write_i16::<LittleEndian>(val).ok(); },
            Value::Uint32(val) => { output.write_u32::<LittleEndian>(val).ok(); },
            Value::Int32(val) => { output.write_i32::<LittleEndian>(val).ok(); },
            Value::Float(val) => { output.write_f32::<LittleEndian>(val).ok(); },
            Value::Double(val) => { output.write_f64::<LittleEndian>(val).ok(); },
          }
        },
    }
}

fn read_field<R: Read>(reader: &mut R, template: &Template) -> Field {
    unimplemented!()
}

fn write_field<W: Write>(writer: W, field: &Field, description: &String) {
    unimplemented!()
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

    // NOTE parse manually to provide better error messages
    for record in lines.deserialize() {
        let line: Rec = record.unwrap();

        let field = to_field(line.typ, line.value);
        info!("{:?}", field);

        write_out(&mut output, field);
    }

    info!("Finished writing to {}", &out_file);
}

fn decode(in_file: &String, out_file: &String, template_file: &String) {
    let mut input;

    match File::open(&in_file) {
        Ok(file_handle) => input = file_handle,
        Err(_) => {
            error!("Could not open input file '{}'!", &in_file);
            return;
        },
    }

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

    // NOTE parse manually to provide better error messages
    for record in lines.deserialize() {
        let template: Template = record.unwrap();

        let field = read_field(&mut input, &template);
        info!("{:?}", field);

        write_field(&mut output, &field, &template.description);
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

        Opt::Decode { in_file, out_file, template_file } => {
            decode(&in_file, &out_file, &template_file);
        },
    }

}

