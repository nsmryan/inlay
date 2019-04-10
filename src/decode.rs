use std::io::BufReader;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use byteorder::ReadBytesExt;

use crate::types::*;
use crate::bit_buffer::*;


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

pub fn decode(in_file: &String, out_file: &String, template_file: &String, repetitions: isize) -> Option<()> {
    let input_file =
        File::open(&in_file).expect(&format!("Could not open input file '{}'!", &in_file));
    let mut input = BufReader::new(input_file);

    let template: File =
        File::open(&template_file).expect(&format!("Could not open template file '{}'!", &template_file));
    let mut lines = csv::Reader::from_reader(&template);
    info!("Opened {}", &template_file);

    let mut output_file =
        File::create(&out_file).expect(&format!("Could not open output file '{}'!", &out_file));


    // Decode template from input file.
    let mut templates: Vec<Template> = vec!();
    for record in lines.records() {
        let rec = record.ok()?;
        let typ = FieldType::from_str(&rec[0]).ok()?;
        let desc = rec[1].to_string();

        let template: Template =
            Template {
                typ: typ,
                description: desc,
            };

        templates.push(template);
    }

    // Decode binary data, writing out to csv file.
    output_file.write_all(&"type,description,value\n".to_string().as_bytes()).unwrap();
    let mut decoder_state = Default::default();
    for _ in 0..repetitions {
        for template in templates.iter() {
            let field = read_field(&mut input, &mut decoder_state, &template)?;

            info!("{}", field);

            write_field(&mut output_file, &field);

            output_file.write_all(&b"\n"[..]).unwrap();
        }
    }

    info!("Finished writing to {}", &out_file);

    Some(())
}
