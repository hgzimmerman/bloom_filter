A few fuzzing targets for use with `cargo-fuzz`.

These were initially used to make sure that the implementations of bloom filters
intended for use across threads were safe for that purpose.


This worked at some point, but I can't seem to get it to use the right panic strategy once I switched from std:: to core:: to enable no_std support.