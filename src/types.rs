use byteorder::{LittleEndian, BigEndian, ByteOrder, WriteBytesExt};
use std::fmt;


/// Rename usize for clarity when dealing with a number of bits.
pub type NumBits = usize;

/// Rename usize for clarity when dealing with a number of bytes.
pub type NumBytes = usize;

/// A field type describes a primitive binary object- either
/// a signed integer, an unsigned integer, or a floating point
/// number (f32 or f64).
#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum FieldType {
    /// Signed integers
    Int(NumBits, Endianness, Option<NumBytes>),

    /// Unsigned integers
    Uint(NumBits, Endianness, Option<NumBytes>),

    /// Single Precision Float
    Float(Endianness),

    /// Double Precision Float
    Double(Endianness),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
          FieldType::Int(num_bits, endianness, num_bytes) => {
              write!(f, "int{}_{}", num_bits, endianness.to_string());
              match num_bytes {
                  None => (),
                  Some(bytes) => write!(f, ":{}", bytes),
              }
          },

          FieldType::Uint(num_bits, endianness, num_bytes) => {
              write!(f, "uint{}_{}", num_bits, endianness.to_string());
              match num_bytes {
                  None => (),
                  Some(bytes) => write!(f, ":{}", bytes),
              }
          },

          FieldType::Float(endianness) => {
              write!(f, "float_{}", endianness.to_string());
          },

          FieldType::Double(endianness) => {
              write!(f, "double_{}", endianness.to_string());
          },
        }
    }
}

impl FieldType {
    /// Get the endianness of a FieldType
    pub fn endianness(&self) -> Endianness {
        match self {
          FieldType::Int(_, endianness, _) => *endianness,

          FieldType::Uint(_, endianness, _) => *endianness,

          FieldType::Float(endianness) => *endianness,

          FieldType::Double(endianness) => *endianness,
        }
    }
}

/// A value is a primitive binary object.
/// These can be 8/16/32/64 bit signed/unsigned integers,
/// of single/double precision floats.
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

/// Endianness as an enum
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

/// A Field is a single entry in a binary file. It consists
/// of the value at a location, a type giving extra information like
/// the endianness and bitwidgth, and a description.
#[derive(PartialEq, Debug, Clone)]
pub struct Field {
    pub value: Value,
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

/// A template gives enough information to decode a field from a binary file,
/// providing the type information used for decoding as well as a description of the
/// field.
#[derive(Eq, PartialEq, Debug, Clone, Deserialize)]
pub struct Template {
  pub typ: FieldType,
  pub description: String,
}

