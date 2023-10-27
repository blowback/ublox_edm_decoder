# u-blox EDM decoder

Takes a hex-encoded dump of EDM data from a u-blox device and prints a human-readable
transcript.

In addition, it can filter out connects/disconnects etc, glom all the data frames together,
and dump them to a file.

## Build it

`cargo build`

## Use it

`cargo run -- data/bad.txt --collect-path FOO`


