use std::fs::File;
use std::io::{Write, Read, Cursor, BufReader};

use byteorder::ReadBytesExt;

use crate::types::*;
use crate::template::*;
use crate::bit_buffer::*;


pub fn decode<W: Write>(in_file: &String, output_file: &mut W, templates: &Vec<Template>, rows: bool) -> Option<()> {
    let input_file =
        File::open(&in_file).expect(&format!("Could not open input file '{}'!", &in_file));
    let mut input = BufReader::new(input_file);

    let template_bytes = templates.num_bits() / 8;
    let mut cursor = Cursor::new(vec![0; template_bytes]);

    // Decode binary data, writing out to csv file.
    if rows {
        output_file.write_all(&"type,description,value\n".to_string().as_bytes()).unwrap();
    } else { // columns
        let descriptions = templates.iter().map(|template| template.description.clone()).collect::<Vec<String>>();
        let header_line = descriptions.join(", ");
        output_file.write_all(header_line.as_bytes()).unwrap();
        output_file.write_all(&b"\n"[..]).unwrap();
    }
    loop {
        let mut decoder_state = Default::default();

        // if we get a read error, we are at the end of input, so just exit cleanly
        if let Err(_) = input.read_exact(cursor.get_mut()) {
            return None;
        }

        for index in 0..templates.len() {
            let template = &templates[index];
            let field = read_field(&mut cursor, &mut decoder_state, &template)?;

            info!("{}", field);

            // for rows, write out type, description, value
            if rows {
                write_field(output_file, &field);
                output_file.write_all(&b"\n"[..]).unwrap();
            } else {
                // for columns, write out value
                output_file.write_all(&format!("{}", field.value.to_string()).as_bytes()).unwrap();

                // only write a ',' if this is not the last entry
                if index != templates.len() - 1 {
                    output_file.write_all(&b", "[..]).unwrap();
                }
            }
        }

        if !rows {
            output_file.write_all(&b"\n"[..]).unwrap();
        }
    }

    Some(())
}

fn read_field<R>(reader: &mut R,
                 bit_buffer: &mut BitBuffer,
                 template: &Template) -> Option<Field>
    where R: ReadBytesExt {

    let value: Value;

    if bit_buffer.is_empty() {
        for _ in 0..template.typ.bit_size().num_bytes() {
            let byte = reader.read_u8().ok()?;

            match template.typ.endianness() {
                Endianness::Little => bit_buffer.push_byte_le(byte)?,
                Endianness::Big    => bit_buffer.push_byte_be(byte)?,
            }
        }
    }

    value = bit_buffer.pull_value(&template.typ)?;

    Some(Field {
        value: value,
        typ: template.typ,
        description: template.description.clone(),
    })
}

fn write_field<W: Write>(writer: &mut W, field: &Field) {
    writer.write_all(&field.to_record().as_bytes()).unwrap();
}

#[test]
fn test_read_field_be() {
    let mut buffer: Vec<u8> = vec!(1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4);

    let descr = "Field".to_string();
    let endianness = Endianness::Big;

    let byte = Template::new(FieldType::u8(endianness), descr.clone());
    let word = Template::new(FieldType::u16(endianness), descr.clone());
    let doubleword = Template::new(FieldType::u32(endianness), descr.clone());
    let quadword = Template::new(FieldType::u64(endianness), descr.clone());

    let field1 = Field::u8(1, endianness, descr.clone());
    let field2 = Field::u16(2, endianness, descr.clone());
    let field3 = Field::u32(3, endianness, descr.clone());
    let field4 = Field::u64(4, endianness, descr.clone());

    let mut cursor = Cursor::new(buffer.as_mut_slice());

    let mut bit_buffer = Default::default();

    assert!(read_field(&mut cursor, &mut bit_buffer, &byte) == Some(field1));
    assert!(read_field(&mut cursor, &mut bit_buffer, &word) == Some(field2));
    assert!(read_field(&mut cursor, &mut bit_buffer, &doubleword) == Some(field3));
    assert!(read_field(&mut cursor, &mut bit_buffer, &quadword) == Some(field4));
}

#[test]
fn test_read_field_le_byte() {
    let mut buffer: Vec<u8> = vec!(1, 2, 0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0);

    let descr = "Field".to_string();
    let endianness = Endianness::Little;
    let byte = Template::new(FieldType::u8(endianness), descr.clone());
    let word = Template::new(FieldType::u16(endianness), descr.clone());
    let doubleword = Template::new(FieldType::u32(endianness), descr.clone());
    let quadword = Template::new(FieldType::u64(endianness), descr.clone());

    let field1 = Field::u8(1, endianness, descr.clone());
    let field2 = Field::u16(2, endianness, descr.clone());
    let field3 = Field::u32(3, endianness, descr.clone());
    let field4 = Field::u64(4, endianness, descr.clone());

    let mut cursor = Cursor::new(buffer.as_mut_slice());

    let mut bit_buffer = Default::default();

    assert!(read_field(&mut cursor, &mut bit_buffer, &byte) == Some(field1));
    assert!(read_field(&mut cursor, &mut bit_buffer, &word) == Some(field2));
    assert!(read_field(&mut cursor, &mut bit_buffer, &doubleword) == Some(field3));
    assert!(read_field(&mut cursor, &mut bit_buffer, &quadword) == Some(field4));
}


#[test]
fn test_read_field_le_bitfields_byte() {
    let byte = 0xA5;

    let mut buffer: Vec<u8> = vec!(byte);

    let descr = "Field".to_string();
    let endianness = Endianness::Little;

    let mut cursor = Cursor::new(buffer.as_mut_slice());

    let typ = FieldType::Uint(1, endianness, BitSize::Bits8);

    let mut bit_buffer = Default::default();

    for index in 0..8 {
        let field = read_field(&mut cursor,
                               &mut bit_buffer,
                               &Template::new(typ, descr.clone()));
        assert_eq!(field, Some(Field { value: Value::Uint8((byte >> index) & 1), typ: typ, description: descr.clone() }));
    }
}

#[test]
fn test_read_field_be_bitfields_word() {
    let mut buffer: Vec<u8> = vec!(0x12, 0x34, 0x56, 0x78);

    let descr = "Field".to_string();
    let endianness = Endianness::Big;

    let mut cursor = Cursor::new(buffer.as_mut_slice());

    let mut bit_buffer = Default::default();

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(4, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(1),
                                   typ: FieldType::Uint(4, endianness, BitSize::Bits16),
                                   description: descr.clone() }));

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(8, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(0x23),
                                   typ: FieldType::Uint(8, endianness, BitSize::Bits16),
                                   description: descr.clone() }));

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(2, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(0x01),
                                   typ: FieldType::Uint(2, endianness, BitSize::Bits16),
                                   description: descr.clone() }));

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(2, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(0x00),
                                   typ: FieldType::Uint(2, endianness, BitSize::Bits16),
                                   description: descr.clone() }));
}

#[test]
fn test_read_field_le_bitfields_word() {
    let mut buffer: Vec<u8> = vec!(0x35, 0x12, 0x56, 0x78);

    let descr = "Field".to_string();
    let endianness = Endianness::Little;

    let mut cursor = Cursor::new(buffer.as_mut_slice());

    let mut bit_buffer = Default::default();

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(4, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(5),
                                   typ: FieldType::Uint(4, endianness, BitSize::Bits16),
                                   description: descr.clone() }));

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(8, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(0x23),
                                   typ: FieldType::Uint(8, endianness, BitSize::Bits16),
                                   description: descr.clone() }));

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(2, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(0x01),
                                   typ: FieldType::Uint(2, endianness, BitSize::Bits16),
                                   description: descr.clone() }));

    let field = read_field(&mut cursor,
                           &mut bit_buffer,
                           &Template::new(FieldType::Uint(2, endianness, BitSize::Bits16), descr.clone()));
    assert_eq!(field, Some(Field { value: Value::Uint8(0x00),
                                   typ: FieldType::Uint(2, endianness, BitSize::Bits16),
                                   description: descr.clone() }));
}

