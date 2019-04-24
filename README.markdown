# Inlay

This program provides a very simple translation from binary data to csv, and csv
to binary data. It is intended for working with simple, fixed size binary formats, and especially
for initial testing and development while there may not be tools specific to the format.

## Template Files
The 'inlay' program makes use a template files, which are just csv files with a name and type for fields.
These files are intended to be simple to write or generate, containing a simple format specifier described below,
and a name to refer to the field.


These files can also contain a 'value' column, and can be encoded into binary without a separate file of
values.

An example template file for a CCSDS Primary Header would look like:
```csv
type,description
uint11_be:16, apid
uint1_be:16, secHeaderFlag
uint1_be:16, type
uint3_be:16, version
uint14_be:16, seqCount
uint2_be:16, seqFlag
uint16_be, packetLen
```

### Types
The types are given by the following regex:
(uint8|uint16|uint32|uint64|int8|int16|int32|int64|float|double)\_(be|le)


Any field can be big endian or little endian, allowing mixed endianness within
a file.


For example, an unsigned, big endian with a width of 16 bits would be 'uint16\_be'.


Note that bit fields are given with an integer width after a colon (unlike in C/C++), so a
3 bit integer within a 16 bit bitfield would be 'uint3\_be:16'.


## CSV -> Binary
When building a binary file, the data to encode can be provided as either a 'row-based' or 'column-based'
csv file.

### Row Based
A row based csv file in the following format must be provided:
```csv
type,description,value
uint8\_be,field1,1
uint16\_be,field2,141
uint32\_be,field3,1245
```


The header must have fields typ, description, and value, in that order.

### Column Based
A column based csv file provides a header with fields for each field of the binary structure,
and must have a template file:
```csv
type,description,value
uint8\_be,field1,1
uint16\_be,field2,141
uint32\_be,field3,1245
```

Note that if a 'value' column is present it is ignored. This template file is used to match field
names to the field in the data file, and gives the types to use when encoding its data.

## Binary -> CSV
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
There are several ways to run the 'inlay' tool. The main way is to either encode or decode a 
series of records, such as decoding a binary file containing one or more record, or encoding
a csv file containing a record of data for each line.


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

### Decode
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
