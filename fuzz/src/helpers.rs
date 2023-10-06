use std::{fs::OpenOptions, io::Write, ops::Mul};

pub fn big_to_pow_18<V: Into<num_bigint::BigUint>>(num: V) -> num_bigint::BigUint {
    num_bigint::BigUint::from(10u32).pow(18).mul(num.into())
}

pub fn write(v: String) {
    let mut fuzz_log = OpenOptions::new()
        .append(true)
        .create(true)
        .open("fuzz.log")
        .expect("fuzz.log should exist");

    fuzz_log
        .write_all(format!("{v}\n").as_bytes())
        .expect("log message should be written");
}
