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

    // map the event
    let mut out_actors = Vec::new();
    for (actoridx, actor) in evnt.get_actors().iter().enumerate() {
        let mut out_steps = Vec::new();
        for (stepidx, step) in actor.get_steps().iter().enumerate() {
            let (wait_on_actoridx, wait_on_stepidx) =
                if let Some(wait_on) = evnt.get_waited_on(actoridx as u16, stepidx as u16) {
                    (Some(wait_on.0), Some(wait_on.1))
                } else {
                    (None, None)
                };
            out_steps.push(OutStep {
                longname: step.get_longname().clone(),
                name: step.get_name().clone(),
                thisidx: u16::try_from(stepidx).unwrap(),
                wait_on_actoridx,
                wait_on_stepidx,
            });
        }
        out_actors.push(OutActor {
            name: actor.get_name().clone(),
            steps: out_steps,
            thisidx: u16::try_from(actoridx).unwrap(),
        });
    }
    let out_event = OutEvent {
        actors: out_actors,
        name: evnt.get_name().clone(),
    };
    let jsn = serde_json::to_string_pretty(&out_event).unwrap();
    println!("{}", jsn);
}
