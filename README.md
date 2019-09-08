# Parser & Disassembler for 16bit Windows target

This is a parser and a disassembler for 16bit Windows target, known as New Executable (NE).

## What is NE?

The "New Executable" format is an old format, which old 16bit Windows OSes (1.0 to 3.1x) have used and 32bit OSes have supported through emulation (WoW32).

This is different from PE (Portable Executable) or the plain MZ (DOS) format.

## How to check if a file is in the NE format?

- NE file has the "MZ" magic at the beginning of the file
  - However, the converse doesn't hold. DOS executables and PE executables have "MZ" too.
- NE file has the "NE" magic at the beginning of the "new header", the place of which is indicated by a little endian 32-bit integer at 0x3C from the beginning of the file.
  - DOE executables lacks this "new header" and PE executables have "PE" there.

## How to obtain NE executables?

- Internet Archive [publishes](https://archive.org/details/softwarelibrary_win3) an archive of Windows 3.1-related softwares. Be sure to check if its legal on your country.
- [Vector](https://www.vector.co.jp), a traditional Japanese software (shareware/freeware) library, has [a directory of Windows 3.1 softwares](https://www.vector.co.jp/vpack/filearea/win31/).

## Prerequisites

- Recent Rust compiler

## Usage

```
$ cargo run path/to/something.exe
$ cargo run path/to/something.dll
```
