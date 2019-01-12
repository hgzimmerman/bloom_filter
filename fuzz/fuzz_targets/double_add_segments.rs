//! A fuzzing target that spawns a reference BF, and a threaded BF.
//! It then generates a list of strings and splits them into chucks and adds them to the reference.
//! Then it moves arcs of the threaded BF to two separate threads and then adds those same strings to the threaded bf at the same time.
//! It then checks if each string returns the same output when checked against each BF.
#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate bloom_filter;
use bloom_filter::bloom_filter::BloomFilter;
use bloom_filter::w_lock_bloom_filter::WLockBloomFilter;
use bloom_filter::hash_numbers::One;
use murmur3::murmur3_32::MurmurHasher;
use std::sync::Arc;
use crossbeam;


fuzz_target!(|data: &[u8]| {
    let chunk_size: usize = 6;
    let bf_size = (data.len() / chunk_size) * 5;
    let mut normal_bloom_filter: BloomFilter<&str, One<MurmurHasher>> = BloomFilter::new(bf_size, One::default());
    let w_lock_bloom_filter: WLockBloomFilter<&str, One<MurmurHasher>> = WLockBloomFilter::new(bf_size, One::default());
    let w_lock_bloom_filter: Arc<WLockBloomFilter<&str, One<MurmurHasher>>> = Arc::new(w_lock_bloom_filter);
    let bf1 = w_lock_bloom_filter.clone();
    let bf2 = w_lock_bloom_filter.clone();


    let v: Vec<&str> = data
        .chunks(chunk_size)
        .map(core::str::from_utf8)
        .filter_map(|x| x.ok())
        .map(|segment| {
            normal_bloom_filter.insert(&segment);
            segment
        })
        .collect();


    let v1 = v.clone();
    let v2 = v.clone();

    crossbeam::scope(|scope| {
        scope.spawn(move |_| {
            let bf = bf1;
            v1
                .iter()
                .for_each(|segment| {
                    bf.insert(segment)
                });
        });
        scope.spawn(move |_| {
            let bf = bf2;
            v2
                .iter()
                .for_each(|segment| {
                    bf.insert(segment)
                });
        });
    }).unwrap();

    v
        .into_iter()
        .for_each(|segment| {
            assert_eq!(normal_bloom_filter.contains(&segment), w_lock_bloom_filter.contains(&segment), "both bf should contain the same bit alignments")
        })

});
