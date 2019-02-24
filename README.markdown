#Inlay

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


When decoding, a file many have many entries. The command line options allow repetitions
of a certain number of repeated structures, or as many as necessary to read the whole file.
Structopt seems to require the flag "-r=-1" rather then "-r -1" when specifying negative
numbers.

