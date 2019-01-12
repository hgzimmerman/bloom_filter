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
    let bf_size = if data.len() < 30 {
        100
    } else {
        (data.len() / chunk_size) * 2
    };
    let mut normal_bloom_filter: BloomFilter<&str, One<MurmurHasher>> = BloomFilter::new(bf_size, One::default());
    let w_lock_bloom_filter: WLockBloomFilter<&str, One<MurmurHasher>> = WLockBloomFilter::new(bf_size, One::default());
    let w_lock_bloom_filter = Arc::new(w_lock_bloom_filter);
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


    let v1: Vec<&str> = v.clone().into_iter().enumerate().filter_map(|(index, s)| if index % 2 == 0 {Some(s)} else {None}).collect();
    let v2: Vec<&str> = v.clone().into_iter().enumerate().filter_map(|(index, s)| if index % 2 == 1 {Some(s)} else {None}).collect();

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
        .iter()
        .for_each(|segment| {
            let normal = normal_bloom_filter.contains(segment);
            let thread = w_lock_bloom_filter.contains(segment);
            if normal != thread {
                println!("'{:?}',  normal: {}, thread: {}, full: {:?}", segment,  normal, thread, v);
                println!("{:?}", w_lock_bloom_filter);
                println!("{:?}", normal_bloom_filter);
                assert!(false)
            }
        })

});
