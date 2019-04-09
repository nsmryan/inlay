use std::fmt;


/// Rename usize for clarity when dealing with a number of bits.
pub type NumBits = usize;

/// Rename usize for clarity when dealing with a number of bytes.
pub type NumBytes = usize;

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
#[allow(non_camel_case_types)]
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

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldType::Int(num_bits, endianness, _) => {
                write!(f, "int{}_{}", num_bits, endianness.to_string())
            },

            FieldType::Uint(num_bits, endianness, _) => {
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

