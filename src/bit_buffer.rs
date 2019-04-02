use std::fmt;
use std::mem;
use std::io::Cursor;

use byteorder::{ReadBytesExt, LittleEndian, BigEndian};

use crate::types::*;


const BITS_IN_BUFFER: u8 = 64;

/// A bit buffer is a collection of bits that can be pushed to
/// and pulled from. Care must be taken to use the desired
/// endianess when interacting with the buffer.
#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct BitBuffer {
    pub bits: u64,
    pub bits_avail: u8,
}

impl fmt::Display for BitBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bit_buffer({:08X}, {})", self.bits & self.mask(), self.bits_avail)
    }
}

impl Iterator for BitBuffer {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.byte_aligned() {
            self.pull_byte()
        } else {
            None
        }
    }
}

#[test]
pub fn test_bit_buffer_iter_be() {
    let mut bit_buffer: BitBuffer = Default::default();

    bit_buffer.push_byte_be(1);
    bit_buffer.push_byte_be(2);
    bit_buffer.push_byte_be(3);
    bit_buffer.push_byte_be(4);

    let bytes: [u8; 4] = [1, 2, 3, 4];

    for i in 0..4 {
        let byte = bit_buffer.next().unwrap();
        assert!(bytes[i] == byte);
    }

    assert!(bit_buffer.next() == None);
}

#[test]
pub fn test_bit_buffer_iter_le() {
    let mut bit_buffer: BitBuffer = Default::default();

    bit_buffer.push_byte_le(1);
    bit_buffer.push_byte_le(2);
    bit_buffer.push_byte_le(3);
    bit_buffer.push_byte_le(4);

    let bytes: [u8; 4] = [4, 3, 2, 1];

    for i in 0..4 {
        assert!(bytes[i] == bit_buffer.next().unwrap());
    }

    assert!(bit_buffer.next() == None);
}

#[test]
pub fn test_bit_buffer_byte_aligned() {
    let mut bit_buffer: BitBuffer = Default::default();

    bit_buffer.bits_avail = 0;
    assert!(bit_buffer.byte_aligned() == false);

    bit_buffer.bits_avail = 1;
    assert!(bit_buffer.byte_aligned() == false);

    bit_buffer.bits_avail = 8;
    assert!(bit_buffer.byte_aligned() == true);

    bit_buffer.bits_avail = 16;
    assert!(bit_buffer.byte_aligned() == true);

    bit_buffer.bits_avail = 24;
    assert!(bit_buffer.byte_aligned() == true);

    bit_buffer.bits_avail = BITS_IN_BUFFER;
    assert!(bit_buffer.byte_aligned() == true);

    bit_buffer.bits_avail = 63;
    assert!(bit_buffer.byte_aligned() == false);
}

#[test]
pub fn test_bit_buffer_push_value_be() {
    let mut bit_buffer: BitBuffer = BitBuffer::default();

    bit_buffer.push_value(Value::Uint8(7),  3, Endianness::Big);
    bit_buffer.push_value(Value::Int16(2),  4, Endianness::Big);
    bit_buffer.push_value(Value::Uint16(1), 1, Endianness::Big);

    assert!(bit_buffer.bits == 0xE5);
}

#[test]
pub fn test_bit_buffer_push_value_le() {
    let mut bit_buffer: BitBuffer = BitBuffer::default();

    bit_buffer.push_value(Value::Uint8(1),  3, Endianness::Little);
    bit_buffer.push_value(Value::Uint16(2), 4, Endianness::Little);
    bit_buffer.push_value(Value::Uint16(1), 1, Endianness::Little);

    assert!(bit_buffer.bits == 0x91);
}

#[test]
pub fn test_bit_buffer_pull_value_be() {
    let mut bit_buffer: BitBuffer = BitBuffer::default();

    bit_buffer.push_value(Value::Uint8(7),  3, Endianness::Big);
    bit_buffer.push_value(Value::Int16(2),  4, Endianness::Big);
    bit_buffer.push_value(Value::Uint16(1), 1, Endianness::Big);

    let typ = FieldType::Uint(3, Endianness::Big, BitSize::Bits8);
    assert!(bit_buffer.pull_value(&typ).unwrap() == Value::Uint8(7));

    let typ = FieldType::Int(4, Endianness::Big, BitSize::Bits16);
    assert!(bit_buffer.pull_value(&typ).unwrap() == Value::Int8(2));

    let typ = FieldType::Uint(1, Endianness::Big, BitSize::Bits16);
    assert!(bit_buffer.pull_value(&typ).unwrap() == Value::Uint8(1));
}

#[test]
pub fn test_bit_buffer_pull_value_le() {
    let mut bit_buffer: BitBuffer = BitBuffer::default();

    bit_buffer.push_value(Value::Uint8(7),  3, Endianness::Little);
    bit_buffer.push_value(Value::Int16(2),  4, Endianness::Little);
    bit_buffer.push_value(Value::Uint16(1), 1, Endianness::Little);

    let typ = FieldType::Uint(3, Endianness::Little, BitSize::Bits8);
    assert!(bit_buffer.pull_value(&typ).unwrap() == Value::Uint8(7));

    let typ = FieldType::Int(4, Endianness::Little, BitSize::Bits16);
    assert!(bit_buffer.pull_value(&typ).unwrap() == Value::Int8(2));

    let typ = FieldType::Uint(1, Endianness::Little, BitSize::Bits16);
    assert!(bit_buffer.pull_value(&typ).unwrap() == Value::Uint8(1));
}

impl BitBuffer {
    pub fn byte_aligned(&self) -> bool {
        (self.bits_avail > 0) && (self.bits_avail % 8 == 0)
    }

    pub fn mask(&self) -> u64 {
        2u64.pow(self.bits_avail as u32) - 1
    }

    pub fn push_byte_be(&mut self, byte: u8) -> Option<()> {
        if self.bits_avail + 8 <= BITS_IN_BUFFER {
            self.bits = (self.bits << 8) | (byte as u64);
            self.bits_avail += 8;
            Some(())
        } else {
            None
        }
    }

    pub fn push_byte_le(&mut self, byte: u8) -> Option<()> {
        if self.bits_avail + 8 <= BITS_IN_BUFFER {
            self.bits |= (byte as u64) << self.bits_avail as u64;
            self.bits_avail += 8;
            Some(())
        } else {
            None
        }
    }

    pub fn push_value(&mut self, value: Value, num_bits: NumBits, endianness: Endianness) -> Option<()> {
        if ((BITS_IN_BUFFER - self.bits_avail) as usize) < num_bits {
            None
        } else {
            let mask = 2u64.pow(num_bits as u32) - 1;

            match endianness {
                Endianness::Big => {
                    // make room for the new value
                    self.bits <<= num_bits;

                    match value {
                        Value::Uint8(val)  => self.bits |= (val as u64) & mask,
                        Value::Int8(val)   => self.bits |= (val as u64) & mask,
                        Value::Uint16(val) => self.bits |= (val as u64) & mask,
                        Value::Int16(val)  => self.bits |= (val as u64) & mask,
                        Value::Uint32(val) => self.bits |= (val as u64) & mask,
                        Value::Int32(val)  => self.bits |= (val as u64) & mask,
                        Value::Uint64(val) => self.bits |= (val as u64) & mask,
                        Value::Int64(val)  => self.bits |= (val as u64) & mask,
                        Value::Float(float_val)  => {
                            unsafe {
                                let val = mem::transmute::<&f32, &u32>(&float_val);
                                self.bits |= (*val as u64) & mask;
                            }
                        },
                        Value::Double(double_val) => {
                            unsafe {
                                let val = mem::transmute::<&f64, &u64>(&double_val);
                                self.bits |= (*val as u64) & mask;
                            }
                        },
                    }
                },

                Endianness::Little => {
                    match value {
                        Value::Uint8(val)  => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Int8(val)   => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Uint16(val) => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Int16(val)  => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Uint32(val) => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Int32(val)  => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Uint64(val) => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Int64(val)  => self.bits |= ((val as u64) & mask) << self.bits_avail,
                        Value::Float(float_val)  => {
                            unsafe {
                                let val = mem::transmute::<&f32, &u32>(&float_val);
                                self.bits |= ((*val as u64) & mask) << self.bits_avail;
                            }
                        },
                        Value::Double(double_val) => {
                            unsafe {
                                let val = mem::transmute::<&f64, &u64>(&double_val);
                                self.bits |= ((*val as u64) & mask) << self.bits_avail;
                            }
                        },
                    }
                },
            }

            self.bits_avail += num_bits as u8;

            Some(())
        }
    }

    pub fn pull_value(&mut self, typ: &FieldType) -> Option<Value> {
        let num_bits: u8 = typ.num_bits() as u8;
        let mask = 2u64.pow(num_bits as u32) - 1;

        self.bits_avail -= num_bits;

        let value;
        match typ.endianness() {
            Endianness::Little => {
                value = self.bits & mask;
                self.bits >>= num_bits;
            }
            Endianness::Big => {
                value = (self.bits >> self.bits_avail) & mask;
                // no need to move bits out in big endian
            }
        }

        match typ {
            FieldType::Int(_, _, _) => {
                if num_bits <= 8 {
                    Some(Value::Int8(value as i8))
                } else if num_bits <= 16 {
                    Some(Value::Int16(value as i16))
                } else if num_bits <= 32 {
                    Some(Value::Int32(value as i32))
                } else if num_bits <= BITS_IN_BUFFER {
                    Some(Value::Int64(value as i64))
                } else {
                    None
                }
            }

            FieldType::Uint(_, _, _) => {
                if num_bits <= 8 {
                    Some(Value::Uint8(value as u8))
                } else if num_bits <= 16 {
                    Some(Value::Uint16(value as u16))
                } else if num_bits <= 32 {
                    Some(Value::Uint32(value as u32))
                } else if num_bits <= BITS_IN_BUFFER {
                    Some(Value::Uint64(value as u64))
                } else {
                    None
                }
            }

            FieldType::Float(_) => {
                unsafe {
                    let mut cursor = Cursor::new(mem::transmute::<&u64, &[u8; 8]>(&value));
                    match typ.endianness() {
                        Endianness::Big => Some(Value::Float(cursor.read_f32::<BigEndian>().unwrap())),
                        Endianness::Little => Some(Value::Float(cursor.read_f32::<LittleEndian>().unwrap())),
                    }
                }
            }

            FieldType::Double(_) => {
                unsafe {
                    let mut cursor = Cursor::new(mem::transmute::<&u64, &[u8; 8]>(&value));
                    match typ.endianness() {
                        Endianness::Big => Some(Value::Double(cursor.read_f64::<BigEndian>().unwrap())),
                        Endianness::Little => Some(Value::Double(cursor.read_f64::<LittleEndian>().unwrap())),
                    }
                }
            }
        }
    }

    fn pull_byte(&mut self) -> Option<u8> {
        if self.byte_aligned() {
            self.bits_avail -= 8;

            // we do not need to modify the bits field here. the bits at the top are
            // left as they are, but made unavailable by reducing bits_avail.
            Some((self.bits >> self.bits_avail) as u8)
        } else {
            None
        }
    }
}
