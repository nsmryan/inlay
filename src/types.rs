use byteorder::{LittleEndian, BigEndian, ByteOrder, WriteBytesExt};


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
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Endianness {
    Little,
    Big
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Field {
    pub value: Value,
    pub endianness: Endianness,
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

