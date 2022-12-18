# lz77
implementation of the lz77 compression algorithm in rust

## File format
The file should be treated as groups of two bytes stored in big endian, eg: A0 F8 B3 DB EA FF would be:
(A0F8, B3DB, EAFF), where the first value is offset, second is match length, with the third being the next character
