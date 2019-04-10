use std::fs::File;
use std::io::Read;

use byteorder::WriteBytesExt;

use crate::types::*;
use crate::bit_buffer::*;


fn to_value(typ: FieldType, value_str: &str) -> Value {
  match typ {
    FieldType::Int(num_bits, _, _) => {
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

    FieldType::Uint(num_bits, _, _) => {
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

fn to_field(typ: FieldType, value_str: &str, description: String) -> Field {
    let value = to_value(typ, value_str);
    Field {
        value: value,
        typ: typ,
        description: description,
    }
}

fn write_out<R>(reader: &mut R, field: &Field, bit_buffer: &mut BitBuffer)
    where R: Read + WriteBytesExt {

    bit_buffer.push_value(field.value, field.typ.num_bits(), field.typ.endianness());

    for byte in bit_buffer {
        reader.write(&[byte]).unwrap();
    }
}

pub fn encode(in_file: &String, out_file: &String) -> Option<()> {
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

    let mut bit_buffer: BitBuffer = Default::default();

    for record in lines.records() {
        let rec = record.ok()?;

        let type_str = &rec[0];
        let description = &rec[1];
        let value_str = &rec[2];

        let typ = type_str.parse().ok()?;

        let field = to_field(typ, value_str, description.to_string());
        info!("{}", field);

        write_out(&mut output, &field, &mut bit_buffer);
    }

    info!("Finished writing to {}", &out_file);

    Some(())
}

