This crate provides bloom filter implementations with the ability to customize the hashing methods used.

In addition to a standard bloom filter, there is a counting bloom filter that can provide an estimate 
for the chance of a false positive to occur, as well as a bloom filter that can be efficiently shared across threads.
