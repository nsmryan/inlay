use std::str::FromStr;
use std::fs::File;

use crate::types::*;


/// A template gives enough information to decode a field from a binary file,
/// providing the type information used for decoding as well as a description of the
/// field.
#[derive(Eq, PartialEq, Debug, Clone, Deserialize)]
pub struct Template {
    pub typ: FieldType,
    pub description: String,
}

impl HasNumBits for Template {
    fn num_bits(&self) -> NumBits {
        self.typ.num_bits()
    }
}

impl Template {
    pub fn new(typ: FieldType, descr: String) -> Template {
        Template { typ: typ, description: descr }
    }

    pub fn read_templates(template_file: &String) -> Option<Vec<Template>> {
        let mut templates: Vec<Template> = vec!();

        let template: File =
            File::open(&template_file).expect(&format!("Could not open template file '{}'!", &template_file));

        let mut lines = csv::Reader::from_reader(&template);
        info!("Opened Template File {}", &template_file);

        // Decode template from input file.
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

        Some(templates)
    }
}

