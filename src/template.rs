use std::fmt;
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

#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum TemplateError {
    LineNumber(usize),
    RecordError(usize),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TemplateError::LineNumber(line_number) => {
                write!(f, "Error parsing template line {}", line_number)
            }

            TemplateError::RecordError(line_number) => {
                write!(f, "Error reading record on line {}", line_number)
            }
        }
    }
}

impl Template {
    pub fn new(typ: FieldType, descr: String) -> Template {
        Template { typ: typ, description: descr }
    }

    pub fn read_templates(template_file: &String) -> Result<Vec<Template>, TemplateError> {
        let mut templates: Vec<Template> = vec!();

        let template: File =
            File::open(&template_file).expect(&format!("Could not open template file '{}'!", &template_file));

        let mut lines = csv::Reader::from_reader(&template);
        info!("Opened Template File {}", &template_file);

        // line number, assuming header
        let mut line_number: usize = 2;

        // Decode template from input file.
        for record in lines.records() {
            let rec = record.map_err(|_| TemplateError::RecordError(line_number))?;
            let typ = FieldType::from_str(&rec[0]).map_err(|_| TemplateError::LineNumber(line_number))?;
            let desc = rec[1].to_string();

            let template: Template =
                Template {
                    typ: typ,
                    description: desc,
                };

            templates.push(template);

            line_number += 1;
        }

        return Ok(templates);
    }
}

