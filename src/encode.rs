use std::fs::File;
use std::io::{Write};

use byteorder::WriteBytesExt;

use crate::types::*;
use crate::bit_buffer::*;
use crate::template::*;


pub fn encode<W: Write>(in_file: &String, output_file: &mut W, templates: &Vec<Template>, rows: bool) -> Option<()> {
    let file = File::open(&in_file).or_else(|err| { error!("Could not open input file '{}'!", &in_file);
                                                    Err(err)
                                                   }).ok().unwrap();

    info!("Opened {}", &in_file);

    let mut lines = csv::Reader::from_reader(file);

    let mut bit_buffer: BitBuffer = Default::default();


    // if processing rows, each row contains a field
    if rows {
        for record in lines.records() {
            let rec = record.ok()?;

            let type_str = &rec[0];
            let description = &rec[1];
            let value_str = &rec[2];

            let typ = type_str.parse().ok()?;

            let field = to_field(typ, value_str, description.to_string());
            info!("{}", field);

            write_out(output_file, &field, &mut bit_buffer);
        }
    } else { // if processing columns, each row contains all items in the template
        for record in lines.records() {
            let rec = record.ok()?;

            for (value_str, template) in rec.iter().zip(templates) {
                let field = to_field(template.typ, value_str, template.description.clone());
                info!("{}", field);

                write_out(output_file, &field, &mut bit_buffer);
            }
        }
    }

    Some(())
}

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

fn write_out<W>(writer: &mut W, field: &Field, bit_buffer: &mut BitBuffer)
    where W: WriteBytesExt {

    bit_buffer.push_value(field.value, field.typ.num_bits(), field.typ.endianness());

    for byte in bit_buffer {
        writer.write(&[byte]).unwrap();
    }
}

#[test]
fn test_encode_to_value() {
    assert_eq!(to_value(FieldType::u8(Endianness::Big), "1"), Value::Uint8(1));
    assert_eq!(to_value(FieldType::u16(Endianness::Big), "1"), Value::Uint16(1));
    assert_eq!(to_value(FieldType::u32(Endianness::Big), "1"), Value::Uint32(1));
    assert_eq!(to_value(FieldType::u64(Endianness::Big), "1"), Value::Uint64(1));
    assert_eq!(to_value(FieldType::i8(Endianness::Big), "1"), Value::Int8(1));
    assert_eq!(to_value(FieldType::i16(Endianness::Big), "1"), Value::Int16(1));
    assert_eq!(to_value(FieldType::i32(Endianness::Big), "1"), Value::Int32(1));
    assert_eq!(to_value(FieldType::i64(Endianness::Big), "1"), Value::Int64(1));
    assert_eq!(to_value(FieldType::float(Endianness::Big), "1.0"), Value::Float(1.0));
    assert_eq!(to_value(FieldType::double(Endianness::Big), "1.0"), Value::Double(1.0));

    assert_eq!(to_value(FieldType::u8(Endianness::Little), "1"), Value::Uint8(1));
    assert_eq!(to_value(FieldType::u16(Endianness::Little), "1"), Value::Uint16(1));
    assert_eq!(to_value(FieldType::u32(Endianness::Little), "1"), Value::Uint32(1));
    assert_eq!(to_value(FieldType::u64(Endianness::Little), "1"), Value::Uint64(1));
    assert_eq!(to_value(FieldType::i8(Endianness::Little), "1"), Value::Int8(1));
    assert_eq!(to_value(FieldType::i16(Endianness::Little), "1"), Value::Int16(1));
    assert_eq!(to_value(FieldType::i32(Endianness::Little), "1"), Value::Int32(1));
    assert_eq!(to_value(FieldType::i64(Endianness::Little), "1"), Value::Int64(1));
    assert_eq!(to_value(FieldType::float(Endianness::Little), "1.0"), Value::Float(1.0));
    assert_eq!(to_value(FieldType::double(Endianness::Little), "1.0"), Value::Double(1.0));
} 
