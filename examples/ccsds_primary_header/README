CCSDS Primary Header Example
============================
This directory contains a definition for the CCSDS Primary Header. It can be
encoded to binary with:

```bash
inlay encode ccsds.csv ccsds_headers.csv -o ccsds.bin
```

resulting a binary file called 'ccsds.bin'.


The file ccsds_row_based.csv shows how inlay can be used with a tranposed csv
file format, where each row describes a single field. This example can be run
with:
```bash
inlay encode ccsds_row_based.csv -o ccsds.bin -r
```

resulting in an identical binary file.


Regardless of how it is created, the binary file ccsds.bin can be decoded
into csv with:
```bash
inlay decode ccsds.csv ccsds.bin
```

to get an output file ccsds.bin.csd, or it can be decoded into a particular file
name as:
```bash
inlay decode ccsds.csv ccsds.bin -o ccsds_output.csv
```


It can also be decoded into a row-based format with
```bash
inlay decode ccsds.csv ccsds.bin -o ccsds_output.csv -r
```


