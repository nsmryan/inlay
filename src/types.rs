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
    Int(NumBits, Endianness),

    /// Unsigned integers
    Uint(NumBits, Endianness),

    /// Single Precision Float
    Float(Endianness),

    /// Double Precision Float
    Double(Endianness),
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
          FieldType::Int(num_bits, endianness) => {
              write!(f, "int{}_{}", num_bits, endianness.to_string())
          },

          FieldType::Uint(num_bits, endianness) => {
              write!(f, "uint{}_{}", num_bits, endianness.to_string())
          },

          FieldType::Float(endianness) => {
              write!(f, "float_{}", endianness.to_string())
          },

          FieldType::Double(endianness) => {
              write!(f, "double_{}", endianness.to_string())
          },
        }
    }
}

impl FieldType {
    /// Get the endianness of a FieldType
    pub fn endianness(&self) -> Endianness {
        match self {
          FieldType::Int(_, endianness) => *endianness,

          FieldType::Uint(_, endianness) => *endianness,

          FieldType::Float(endianness) => *endianness,

          FieldType::Double(endianness) => *endianness,
        }
    }

    pub fn num_bits(&self) -> NumBits {
        match self {
          FieldType::Int(num_bits, _) => *num_bits,

          FieldType::Uint(num_bits, _) => *num_bits,

          FieldType::Float(_) => 32,

          FieldType::Double(_) => 64,
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

  pub fn num_bits(&self) -> usize {
      self.num_bytes() * 8
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

impl Default for Endianness {
    fn default() -> Self {
        Endianness::Little
    }
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

    pub fn full_width(&self) -> bool {
        match self.typ {
          FieldType::Int(num_bits, _) => num_bits == self.value.num_bits(),

          FieldType::Uint(num_bits, _) => num_bits == self.value.num_bits(),

          FieldType::Float(_) => true,

          FieldType::Double(_) => true,
        }
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


#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct BitBuffer {
    pub bits: u64,
    pub bits_avail: u8,
    pub endianness: Endianness,
}

impl BitBuffer {
    pub fn push_byte_be(&mut self, byte: u8) -> Option<()> {
        if self.bits_avail + 8 < 64 {
            self.bits = ((self.bits as u64) << 8) | byte as u64;
            self.bits_avail += 8;
            Some(())
        } else {
            None
        }
    }

    pub fn push_byte_le(&mut self, byte: u8) -> Option<()> {
        if self.bits_avail + 8 < 64 {
            self.bits |= ((byte as u64) << self.bits_avail as u64);
            self.bits_avail += 8;
            Some(())
        } else {
            None
        }
    }

    pub fn push_value(&mut self, value: Value, num_bits: NumBits, endianness: Endianness) {
        match value {
          Value::Uint8(val)  => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Int8(val)   => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Uint16(val) => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Int16(val)  => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Uint32(val) => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Int32(val)  => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Uint64(val) => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Int64(val)  => {
              self.bits = (self.bits << num_bits) | (val as u64);
          },

          Value::Float(val)  => {
              match endianness {
                  Endianness::Little => {
                      unimplemented!();
                  },

                  Endianness::Big => {
                      unimplemented!();
                  },
              }
          },

          Value::Double(val) => {
              match endianness {
                  Endianness::Little => {
                      unimplemented!();
                  },

                  Endianness::Big => {
                      unimplemented!();
                  },
              }
          },
        }

        self.bits_avail += num_bits as u8;
    }

    // NOTE this could provide an Option<Value> to indicate errors
    pub fn pull_value_int(&mut self, num_bits: u8) -> Value {
        let value = self.bits & ((num_bits as u64).pow(2) - 1);
        self.bits >>= num_bits;

        if num_bits <= 8 {
            Value::Int8(value as i8)
        } else if num_bits <= 16 {
            Value::Int16(value as i16)
        } else if num_bits <= 32 {
            Value::Int32(value as i32)
        } else if num_bits <= 64 {
            Value::Int64(value as i64)
        } else {
            panic!("{} bits in a field are not supported!");
        }
    }

    // NOTE this could provide an Option<Value> to indicate errors
    pub fn pull_value_uint(&mut self, num_bits: u8) -> Value {
        let value = self.bits & ((num_bits as u64).pow(2) - 1);
        self.bits >>= num_bits;

        if num_bits <= 8 {
            Value::Uint8(value as u8)
        } else if num_bits <= 16 {
            Value::Uint16(value as u16)
        } else if num_bits <= 32 {
            Value::Uint32(value as u32)
        } else if num_bits <= 64 {
            Value::Uint64(value as u64)
        } else {
            panic!("{} bits in a field are not supported!");
        }
    }

    pub fn pull_byte(&mut self) -> u8 {
        let result_byte;

        if self.bits_avail < 8 {
            panic!("No byte available to pull!");
        } else {
            match self.endianness {
                Endianness::Little => {
                    let byte: u8 = self.bits as u8;
                    self.bits = self.bits >> 8;
                    result_byte = byte;
                },

                Endianness::Big => {
                    let byte: u8 = (self.bits >> (self.bits_avail - 8)) as u8;
                    // we do not need to modify the bits field here. the bits at the top are
                    // left as they are, but made unavailble by reducing bits_avail.
                    result_byte = byte;
                },
            }

            self.bits_avail -= 8;
        }

        return result_byte;
    }
}
