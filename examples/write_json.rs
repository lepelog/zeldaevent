use std::env;
use std::fs::File;
use std::io::Read;

extern crate serde;
use serde::Serialize;
extern crate serde_json;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutStep {
    longname: String,
    name: String,
    thisidx: u16,
    wait_on_actoridx: Option<u16>,
    wait_on_stepidx: Option<u16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutActor {
    name: String,
    thisidx: u16,
    steps: Vec<OutStep>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutEvent {
    name: String,
    actors: Vec<OutActor>,
}

use zeldaevent::zevfile::parse_zev;

pub fn main() {
    let filename = env::args().skip(1).next().expect("no filename");
    let eventname = env::args().skip(2).next().expect("no eventname");
    let mut file = File::open(filename).expect("file not found");
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let zev = parse_zev(buf.as_slice()).unwrap();
    // println!("{:?}", zev);
    let evnt = zev
        .into_iter()
        .find(|e| e.get_name() == &eventname)
        .expect("event not found");
    println!("{}", evnt.to_json().unwrap());
}
