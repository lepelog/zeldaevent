use std::env;
use std::fs::File;
use std::io::Read;

use zeldaevent::zevfile::{parse_zev, write_zev};

pub fn main() {
    let filename = env::args().skip(1).next().unwrap();
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let zev = parse_zev(buf.as_slice()).unwrap();
    // println!("{:?}", zev);
    let written = write_zev(&zev).unwrap();
    assert_eq!(buf, written);
}
