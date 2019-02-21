use byteorder::{LittleEndian, BigEndian, ByteOrder, WriteBytesExt};
use std::fmt;


#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum FieldType {
  uint8_be,
  int8_be,
  uint16_be,
  int16_be,
  uint32_be,
  int32_be,
  float_be,
  double_be,
  uint8_le,
  int8_le,
  uint16_le,
  int16_le,
  uint32_le,
  int32_le,
  float_le,
  double_le,
}

impl FieldType {
    pub fn endianness(&self) -> Endianness {
        match self {
          FieldType::uint8_be  |
          FieldType::int8_be   |
          FieldType::uint16_be |
          FieldType::int16_be  |
          FieldType::uint32_be |
          FieldType::int32_be  |
          FieldType::float_be  |
          FieldType::double_be => Endianness::Big,

          FieldType::uint8_le  |
          FieldType::int8_le   |
          FieldType::uint16_le |
          FieldType::int16_le  |
          FieldType::uint32_le |
          FieldType::int32_le  |
          FieldType::float_le  |
          FieldType::double_le => Endianness::Little,
        }
    }

    fn to_string(&self) -> String {
        match self {
          FieldType::uint8_be => "uint8_be".to_string(),
          FieldType::int8_be => "int8_be".to_string(),
          FieldType::uint16_be => "uint16_be".to_string(),
          FieldType::int16_be => "int16_be".to_string(),
          FieldType::uint32_be => "uint32_be".to_string(),
          FieldType::int32_be => "int32_be".to_string(),
          FieldType::float_be => "float_be".to_string(),
          FieldType::double_be => "double_be".to_string(),

          FieldType::uint8_le => "uint8_le".to_string(),
          FieldType::int8_le => "int8_le".to_string(),
          FieldType::uint16_le => "uint16_le".to_string(),
          FieldType::int16_le => "int16_le".to_string(),
          FieldType::uint32_le => "uint32_le".to_string(),
          FieldType::int32_le => "int32_le".to_string(),
          FieldType::float_le => "float_le".to_string(),
          FieldType::double_le => "double_le".to_string(),
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum Value {
  Uint8(u8),
  Int8(i8),
  Uint16(u16),
  Int16(i16),
  Uint32(u32),
  Int32(i32),
  Float(f32),
  Double(f64),
}

impl Value {
  pub fn num_bytes(&self) -> usize {
    match self {
      Value::Uint8(_)  => 1,
      Value::Int8(_)   => 1,
      Value::Uint16(_) => 2,
      Value::Int16(_)  => 2,
      Value::Uint32(_) => 4,
      Value::Int32(_)  => 4,
      Value::Float(_)  => 4,
      Value::Double(_) => 8,
    }
  }

  pub fn to_string(&self) -> String {
    match self {
      Value::Uint8(val)  => format!("{}", val),
      Value::Int8(val)   => format!("{}", val),
      Value::Uint16(val) => format!("{}", val),
      Value::Int16(val)  => format!("{}", val),
      Value::Uint32(val) => format!("{}", val),
      Value::Int32(val)  => format!("{}", val),
      Value::Float(val)  => format!("{}", val),
      Value::Double(val) => format!("{}", val),
    }
  }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Endianness {
    Little,
    Big
}

#[derive(PartialEq, Debug, Clone)]
pub struct Field {
    pub value: Value,
    pub endianness: Endianness,
    pub typ: FieldType,
    pub description: String,
}

impl Field {
    pub fn to_record(&self) -> String {
        format!("{},{},{}", self.typ.to_string(), self.description, self.value.to_string())
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.typ.to_string(), self.description, self.value.to_string())
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize)]
pub struct Rec {
  pub typ: FieldType,
  pub value: String,
  pub description: String,
}

#[derive(Eq, PartialEq, Debug, Clone, Deserialize)]
pub struct Template {
  pub typ: FieldType,
  pub description: String,
}

