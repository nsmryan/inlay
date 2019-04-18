Inlay
=====

This program provides a very simple translation from binary data to csv, and csv
to binary data. It is intended for working with simple binary formats, and especially
for initial testing and development while there may not be tools specific to the format.

CSV -> Binary
When building a binary file, a csv file in the following format must be provided:
type,description,value
uint8\_be,field1,1
uint16\_be,field2,141
uint32\_be,field3,1245


The header must have fields typ, description, and value, in that order. The types
are given by the following regex:
(uint8|uint16|uint32|uint64|int8|int16|int32|int64|float|double)\_(be|le)

Any field can be big endian or little endian, allowing mixed endianness within
a file.

Binary -> CSV
When translating binary to CSV, a file of the following format must be provided:
type,description
uint64\_be,a 64 bit field
uint8\_be,an 8 bit field

This file must have a header "type,description" and any number of entries with
a type as defined above. The description is optional and will be copied into the
output csv file.

The output csv file is in exactly the same format as the input csv file when 
encoding from CSV -> Binary. This means that a binary structure can be decoded,
modified and written back.
To assist with this use case, if the "template" file given during decoding has
a "values" column it will be ignored. This allows a csv file from decoding to be used as the
template when decoding other instances of a binary structure.


## Usage
### Encode 
Encode a single file into a binary file, row format:
  * inlay encode template.csv 
  * inlay encode template.csv -o data.bin

Encode a column csv file into a binary file:
  * inlay encode template.csv data.csv
  * inlay encode template.csv data.csv -o data.bin

Encode multiple input files into their own binary files:
  * inlay encode template.csv data.csv
  * inlay encode template.csv data.csv data2.csv data3.csv

Encode multiple input files into a single binary file:
  * inlay encode template.csv data.csv
  * inlay encode template.csv data.csv data2.csv data3.csv -o data.bin

Decode a single binary file, row format:
  * inlay decode template.csv data.bin -r

Decode a single binary file, col format:
  * inlay decode template.csv data.bin

Decode multiple binary files, row format:
  * inlay decode template.csv data.bin data2.bin data3.bin -r

Decode multiple binary files, col format:
  * inlay decode template.csv data.bin data2.bin data3.bin -r

Decode multiple binary files into a single file, row format:
  * inlay decode template.csv data.bin data2.bin data3.bin -r -o output.csv

Decode multiple binary files into a single file, col format:
  * inlay decode template.csv data.bin data2.bin data3.bin -o output.csv
