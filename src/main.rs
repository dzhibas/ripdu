#![feature(integer_atomics)]
#![feature(exclusive_range_pattern)]

extern crate ignore;
extern crate human_size;
extern crate num_cpus;

use ignore::WalkBuilder;
use std::sync::atomic::{AtomicU64, Ordering};
use ignore::WalkState::*;
use std::sync::{Arc, Mutex};
use std::env::args;
use std::collections::HashMap;
use human_size::{Size, Multiple};

fn main() {
    let dir = match args().skip(1).next() {
        Some(x) => x,
        _ => ".".to_string(),
    };
    let acc = Arc::new(AtomicU64::new(0));
    let files = Arc::new(Mutex::new(HashMap::new()));
    let _ = WalkBuilder::new(dir)
        .ignore(false)
        .threads(num_cpus::get())
        .git_global(false)
        .git_ignore(false)
        .git_exclude(false)
        .parents(false)
        .hidden(false)
        .build_parallel()
        .run(|| {
            let acc_in = Arc::clone(&acc);
            let files_in = Arc::clone(&files);
            Box::new(move |result| {
                let pp = match result {
                    Ok(result) => result,
                    Err(_) => return Continue,
                };

                let p = pp.path();
                let d = p.to_path_buf().into_os_string().into_string().unwrap();
                if let Some(ref result_type) = pp.file_type() {
                    if result_type.is_file() && !result_type.is_symlink() {
                        match pp.metadata() {
                            Ok(m) => {
                                // add into verbose hashmap only if size more than 10Mb
                                if m.len() > 1048576u64 {
                                    let mut data = files_in.lock().unwrap();
                                    data.insert(d, m.len());
                                }
                                acc_in.fetch_add(m.len(), Ordering::SeqCst);
                            },
                            Err(_) => return Continue,
                        };
                    };
                } else {
                    return Continue;
                }
                
                Continue
            })
        });

    let res = Arc::try_unwrap(acc).unwrap().into_inner();
    let all = Arc::try_unwrap(files).unwrap().into_inner().unwrap();

    // for (_file, _size) in all.iter() {
        // println!("{0:<020} {1}", get_human_readable_name(*_size), _file);
    // }

    let mut count_vec: Vec<_> = all.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for x in 0..10 {
        println!("{} {}", get_human_readable_name(*count_vec[x].1), count_vec[x].0);
    }
    println!("Total size of folder:\n {}", get_human_readable_name(res));
}

fn get_human_readable_name(size_in_bytes: u64) -> Size {
    const KILO:u64 = 1024u64;
    const MEGA:u64 = 1048576u64;
    const GIGA:u64 = 1073741824u64;
    const MAX:u64 = <u64>::max_value();

    match size_in_bytes {
        0..KILO => Size::new(_round(size_in_bytes as f64), Multiple::Byte).unwrap(),
        KILO..MEGA => Size::new(_divide_and_round(size_in_bytes, KILO), Multiple::Kilobyte).unwrap(),
        MEGA..GIGA => Size::new(_divide_and_round(size_in_bytes, MEGA), Multiple::Megabyte).unwrap(),
        GIGA..MAX => Size::new(_divide_and_round(size_in_bytes, GIGA), Multiple::Gigabyte).unwrap(),
        _ =>  Size::new(_round(size_in_bytes as f64), Multiple::Byte).unwrap(),
    }
}

fn _divide_and_round(size: u64, divisor: u64) -> f64 {
    _round(size as f64 / divisor as f64)
}
fn _round(what: f64) -> f64 {
    (what * 100.0).round() / 100.0
}