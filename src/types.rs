use byteorder::{LittleEndian, BigEndian, ByteOrder, WriteBytesExt};
use std::fmt;


pub type NumBits = usize;

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum FieldType {
    Int(NumBits, Endianness),
    Uint(NumBits, Endianness),
    Float(Endianness),
    Double(Endianness),
}

impl FieldType {
    pub fn endianness(&self) -> Endianness {
        match self {
          FieldType::Int(_, endianness) => *endianness,

          FieldType::Uint(_, endianness) => *endianness,

          FieldType::Float(endianness) => *endianness,

          FieldType::Double(endianness) => *endianness,
        }
    }

    fn to_string(&self) -> String {
        match self {
          FieldType::Int(num_bits, endianness) => format!("int{}_{}", num_bits, endianness.to_string()),

          FieldType::Uint(num_bits, endianness) => format!("uint{}_{}", num_bits, endianness.to_string()),

          FieldType::Float(endianness) => format!("float_{}", endianness.to_string()),

          FieldType::Double(endianness) => format!("double_{}", endianness.to_string()),
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
  Uint64(u64),
  Int64(i64),
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
      Value::Uint64(_) => 8,
      Value::Int64(_)  => 8,
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
      Value::Uint64(val) => format!("{}", val),
      Value::Int64(val)  => format!("{}", val),
      Value::Float(val)  => format!("{}", val),
      Value::Double(val) => format!("{}", val),
    }
  }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum Endianness {
    Little,
    Big
}

impl Endianness {
    fn to_string(&self) -> String {
        match self {
            Endianness::Little => "le".to_string(),
            Endianness::Big => "be".to_string(),
        }
    }
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

