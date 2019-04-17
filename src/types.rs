use std::fmt;
use std::str::FromStr;
use std::vec::*;
use std::iter::*;

use regex::Regex;


/// Rename usize for clarity when dealing with a number of bits.
pub type NumBits = usize;

/// Rename usize for clarity when dealing with a number of bytes.
pub type NumBytes = usize;

pub trait HasNumBits {
    fn num_bits(&self) -> NumBits;
}

impl<A: HasNumBits> HasNumBits for Vec<A> {
    fn num_bits(&self) -> NumBits {
        self.iter().fold(0, |sum, value| value.num_bits() + sum)
    }
}

/// Signedness is used to indicate whether an integer is signed
/// or unsigned.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum Signedness {
    Unsigned,
    Signed,
}

/// A field type describes a primitive binary object- either
/// a signed integer, an unsigned integer, or a floating point
/// number (f32 or f64).
///
/// A field type also carries information about the memory
/// area that contains it. This is used for bitfields to indicate
/// the size of the bit field that contains a particular set of bits.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Deserialize)]
pub enum FieldType {
    /// Signed integers
    Int(NumBits, Endianness, BitSize),

    /// Unsigned integers
    Uint(NumBits, Endianness, BitSize),

    /// Single Precision Float
    Float(Endianness),

    /// Double Precision Float
    Double(Endianness),
}

impl HasNumBits for FieldType {
    fn num_bits(&self) -> NumBits {
        match self {
            FieldType::Int(num_bits, _, _) => *num_bits,
            FieldType::Uint(num_bits, _, _) => *num_bits,
            FieldType::Float(_) => 32,
            FieldType::Double(_) => 64,
        }
    }
}

impl FieldType {
    pub fn u8(endianness: Endianness) -> FieldType {
        FieldType::Uint(8, endianness, BitSize::Bits8)
    }

    pub fn u16(endianness: Endianness) -> FieldType {
        FieldType::Uint(16, endianness, BitSize::Bits16)
    }

    pub fn u32(endianness: Endianness) -> FieldType {
        FieldType::Uint(32, endianness, BitSize::Bits32)
    }

    pub fn u64(endianness: Endianness) -> FieldType {
        FieldType::Uint(64, endianness, BitSize::Bits64)
    }

    pub fn i8(endianness: Endianness) -> FieldType {
        FieldType::Int(8, endianness, BitSize::Bits8)
    }

    pub fn i16(endianness: Endianness) -> FieldType {
        FieldType::Int(16, endianness, BitSize::Bits16)
    }

    pub fn i32(endianness: Endianness) -> FieldType {
        FieldType::Int(32, endianness, BitSize::Bits32)
    }

    pub fn i64(endianness: Endianness) -> FieldType {
        FieldType::Int(64, endianness, BitSize::Bits64)
    }

    pub fn float(endianness: Endianness) -> FieldType {
        FieldType::Float(endianness)
    }

    pub fn double(endianness: Endianness) -> FieldType {
        FieldType::Double(endianness)
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::Int(num_bits, endianness, bit_size) => {
                if *num_bits != bit_size.num_bits() {
                    write!(f, "int{}_{}:{}", num_bits, endianness.to_string(), bit_size.num_bits())
                }
                else
                {
                    write!(f, "int{}_{}", num_bits, endianness.to_string())
                }
            },

            FieldType::Uint(num_bits, endianness, bit_size) => {
                if *num_bits != bit_size.num_bits() {
                    write!(f, "uint{}_{}:{}", num_bits, endianness.to_string(), bit_size.num_bits())
                } else
                {
                    write!(f, "uint{}_{}", num_bits, endianness.to_string())
                }
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
            FieldType::Int(_, endianness, _) => *endianness,

            FieldType::Uint(_, endianness, _) => *endianness,

            FieldType::Float(endianness) => *endianness,

            FieldType::Double(endianness) => *endianness,
        }
    }

    pub fn num_bits(&self) -> NumBits {
        match self {
            FieldType::Int(num_bits, _, _) => *num_bits,

            FieldType::Uint(num_bits, _, _) => *num_bits,

            FieldType::Float(_) => 32,

            FieldType::Double(_) => 64,
        }
    }

    pub fn bit_size(&self) -> BitSize {
        match self {
            FieldType::Int(_, _, bit_size) => *bit_size,

            FieldType::Uint(_, _, bit_size) => *bit_size,

            FieldType::Float(_) => BitSize::Bits32,

            FieldType::Double(_) => BitSize::Bits64,
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldParseError(());

impl FromStr for FieldType {
    type Err = FieldParseError;

    fn from_str(type_str: &str) -> Result<FieldType, FieldParseError> {
        lazy_static! {
          static ref TYPE_REGEX: Regex =
              Regex::new(r"(float|double|int|uint)(\d{0,2})_(be|le)(:8|:16|:32|:64)?").unwrap();
        }

        let type_str = type_str.to_lowercase();

        let matches = TYPE_REGEX.captures(&type_str).ok_or(FieldParseError(()))?;

        match &matches[1] {
            "uint" => {
                let num_bits = matches[2].parse::<NumBits>().or(Err(FieldParseError(())))?;

                let within_bits =
                    matches.get(4).map(|mat| BitSize::from_str_bits(&mat.as_str()[1..]))
                                  .unwrap_or(BitSize::fits_within(num_bits));

                match &matches[3] {
                    "be" => Ok(FieldType::Uint(num_bits, Endianness::Big, within_bits)),

                    "le" => Ok(FieldType::Uint(num_bits, Endianness::Little, within_bits)),

                     _ => {
                         error!("Endianness '{}' not expected!", &matches[3]);
                         Err(FieldParseError(()))
                     },
                }
            },

            "int" => {
                let num_bits = matches[2].parse::<NumBits>().or(Err(FieldParseError(())))?;

                let within_bits =
                    matches.get(4).map(|mat| BitSize::from_str_bits(&mat.as_str()[1..]))
                                 .unwrap_or(BitSize::fits_within(num_bits));

                match &matches[3] {
                    "be" => Ok(FieldType::Int(num_bits, Endianness::Big, within_bits)),

                    "le" => Ok(FieldType::Int(num_bits, Endianness::Little, within_bits)),

                     _ => {
                         error!("Endianness '{}' not expected!", &matches[3]);
                         Err(FieldParseError(()))
                     },
                }
            },

            "float" => {
                match &matches[3] {
                    "be" => Ok(FieldType::Float(Endianness::Big)),

                    "le" => Ok(FieldType::Float(Endianness::Little)),

                     _ => {
                         error!("Endianness '{}' not expected!", &matches[3]);
                         Err(FieldParseError(()))
                     },
                }
            },

            "double" => {
                match &matches[3] {
                    "be" => Ok(FieldType::Double(Endianness::Big)),

                    "le" => Ok(FieldType::Double(Endianness::Little)),

                     _ => {
                         error!("Endianness '{}' not expected!", &matches[3]);
                         Err(FieldParseError(()))
                     },
                }
            },

            _ => {
                error!("Type '{}' unexpected in field type '{}'", &matches[1], type_str);
                Err(FieldParseError(()))
            }
        }
    }
}


/// A BitSize is a number of bits for a particular field.
/// This is used when processing bit fields, which are nested
/// inside of a larger structure of a fixed number of bits.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Deserialize)]
pub enum BitSize {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
}

impl BitSize {
    pub fn from_str_bits(chars: &str) -> Self {
        match chars {
            "8"  => BitSize::Bits8,
            "16" => BitSize::Bits16,
            "32" => BitSize::Bits32,
            "64" => BitSize::Bits64,
            _    => panic!("{} chars not supports for BitSize"),
        }
    }

    pub fn num_bytes(&self) -> NumBytes {
        match self {
            BitSize::Bits8  => 1,
            BitSize::Bits16 => 2,
            BitSize::Bits32 => 4,
            BitSize::Bits64 => 8,
        }
    }

    pub fn fits_within(num_bits: NumBits) -> Self {
        if num_bits <= 8 {
            BitSize::Bits8
        } else if num_bits <= 16 {
            BitSize::Bits16
        } else if num_bits <= 32 {
            BitSize::Bits32
        } else if num_bits <= 64 {
            BitSize::Bits64
        } else {
            panic!("Bit sizes of {} not supported", num_bits);
        }
    }
}

impl HasNumBits for BitSize {
    fn num_bits(&self) -> NumBits {
        self.num_bytes() * 8
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

impl HasNumBits for Value {
    fn num_bits(&self) -> NumBits {
        match self {
            Value::Uint8(val)  => 8,
            Value::Int8(val)   => 8,
            Value::Uint16(val) => 16,
            Value::Int16(val)  => 16,
            Value::Uint32(val) => 32,
            Value::Int32(val)  => 32,
            Value::Uint64(val) => 64,
            Value::Int64(val)  => 64,
            Value::Float(val)  => 32,
            Value::Double(val) => 64,
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

    pub fn u8(val: u8, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Uint8(val), typ: FieldType::u8(endianness), description: descr }
    }

    pub fn u16(val: u16, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Uint16(val), typ: FieldType::u16(endianness), description: descr }
    }

    pub fn u32(val: u32, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Uint32(val), typ: FieldType::u32(endianness), description: descr }
    }

    pub fn u64(val: u64, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Uint64(val), typ: FieldType::u64(endianness), description: descr }
    }

    pub fn i8(val: i8, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Int8(val), typ: FieldType::i8(endianness), description: descr }
    }

    pub fn i16(val: i16, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Int16(val), typ: FieldType::i16(endianness), description: descr }
    }

    pub fn i32(val: i32, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Int32(val), typ: FieldType::i32(endianness), description: descr }
    }

    pub fn i64(val: i64, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Int64(val), typ: FieldType::i64(endianness), description: descr }
    }

    pub fn float(val: f32, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Float(val), typ: FieldType::float(endianness), description: descr }
    }

    pub fn double(val: f64, endianness: Endianness, descr: String) -> Field {
        Field { value: Value::Double(val), typ: FieldType::double(endianness), description: descr }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.typ.to_string(), self.description, self.value.to_string())
    }
}

impl HasNumBits for Field {
    fn num_bits(&self) -> NumBits {
        self.typ.num_bits()
    }
}

