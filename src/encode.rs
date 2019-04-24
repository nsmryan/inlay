use std::fs::File;
use std::io::{Write, Read};

use byteorder::WriteBytesExt;

use crate::types::*;
use crate::bit_buffer::*;
use crate::template::*;


pub fn encode<R: Read, W: Write>(input: &mut R, output: &mut W, templates: &Vec<Template>, rows: bool) -> Option<()> {
    let mut lines = csv::Reader::from_reader(input);

    let mut bit_buffer: BitBuffer = Default::default();

    // if processing rows, each row contains a field
    trace!("Starting encoding");
    if rows {
        trace!("Row based");
        let headers: Vec<&str> =
            lines.headers().expect("Expected template file to have a csv file header").iter().collect();

        let required_fields = vec!("value", "description", "typ");
        if !required_fields.iter().all(|field_name| headers.contains(&field_name)) {
            panic!("A row based csv file must at least have a field for value, description and type!");
        }

        for record in lines.records() {
            trace!("Processing record");

            let rec = record.ok()?;

            let type_str = &rec[0];
            let description = &rec[1];
            let value_str = &rec[2];

            let typ = type_str.parse().ok()?;

            let field = Field { value: to_value(typ, value_str),
                                typ: typ,
                                description: description.to_string(),
            };
            trace!("{}", field);

            write_out(output, &field, &mut bit_buffer);
        }
    } else { // if processing columns, each row contains all items in the template
        trace!("Column based");
        for record in lines.records() {
            trace!("Processing record");

            let rec = record.ok()?;

            for (value_str, template) in rec.iter().zip(templates) {
                trace!("Processing field");

                let field = Field { value: to_value(template.typ, value_str),
                                    typ: template.typ,
                                    description: template.description.clone(),
                };
                trace!("{}", field);

                write_out(output, &field, &mut bit_buffer);
            }
        }
    }

    Some(())
}

fn to_value(typ: FieldType, value_str: &str) -> Value {
  let value_str = value_str.trim();

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
    Field {
        value: to_value(typ, value_str),
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
