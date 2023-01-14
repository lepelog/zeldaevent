use std::env;
use std::fs::File;
use std::io::{Read, Write};

use zeldaevent::zevfile::{parse_zev, write_zev, Event, StepDataValues};

fn idx_by_name(evnt: &Event, actor_idx: usize, step_name: &str) -> usize {
    if let Some(idx) = evnt.actors[actor_idx]
        .get_steps()
        .iter()
        .position(|step| step.get_longname() == step_name)
    {
        return idx;
    } else {
        panic!("couln't find {step_name}");
    }
}

pub fn main() {
    let mut file = File::open("../ss-object-map/scripts/sstools/allzev/F200_zev.dat").unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let mut zev = parse_zev(buf.as_slice()).unwrap();
    // println!("{:?}", zev);
    let bridge_event = zev
        .iter_mut()
        .filter(|z| z.get_name() == "F200R02inpa")
        .next()
        .unwrap();
    let camera_idx = bridge_event.get_actoridx_for_name("Camera").unwrap();
    let link_idx = bridge_event.get_actoridx_for_name("Link").unwrap();
    bridge_event.actors[camera_idx].steps[1].data[0].values = StepDataValues::Ints(vec![0]);
    bridge_event.remove_waiting(camera_idx, 1);
    bridge_event.remove_step(link_idx, 3);
    bridge_event.remove_step(link_idx, 2);
    bridge_event.remove_step(camera_idx, 5);
    bridge_event.remove_step(camera_idx, 3);
    let written = write_zev(&zev).unwrap();
    let mut file = File::create("testF200.dat").unwrap();
    file.write(&written).unwrap();
}

fn write_evnt(evnt: &Event, filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write(&evnt.to_dot_file().as_bytes())?;
    Ok(())
}
