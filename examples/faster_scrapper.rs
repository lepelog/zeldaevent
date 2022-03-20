use std::env;
use std::fs::File;
use std::io::{Read, Write};

use zeldaevent::zevfile::{parse_zev, write_zev, Event};

pub fn main() {
    let mut file = File::open("../ss-object-map/scripts/sstools/allzev/F300_zev.dat").unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let mut zev = parse_zev(buf.as_slice()).unwrap();
    // println!("{:?}", zev);
    let mut scrapper_evnt = zev
        .iter_mut()
        .filter(|z| z.get_name() == "SalbageFayCall2")
        .next()
        .unwrap();
    write_evnt(&scrapper_evnt, "scrapper_orig.dot").unwrap();
    // scrapper_evnt.remove_all_waits();
    let scrapper_idx = scrapper_evnt.get_actoridx_for_name("NpcSlrb").unwrap();
    // scrapper_evnt.remove_waiting(scrapper_idx, 1);
    // scrapper_evnt.remove_waiting(scrapper_idx, 2);
    // scrapper_evnt.remove_waiting(scrapper_idx, 3);
    scrapper_evnt.remove_step(scrapper_idx, 12);
    scrapper_evnt.remove_step(scrapper_idx, 11);
    scrapper_evnt.remove_step(scrapper_idx, 10);
    scrapper_evnt.remove_step(scrapper_idx, 6);
    scrapper_evnt.remove_step(scrapper_idx, 4);
    scrapper_evnt.remove_step(scrapper_idx, 3);
    scrapper_evnt.remove_step(scrapper_idx, 2);
    scrapper_evnt.remove_step(scrapper_idx, 1);
    // scrapper_evnt.remove_step(scrapper_idx, 2);
    let camera_idx = scrapper_evnt.get_actoridx_for_name("Camera").unwrap();
    // delete camera pans
    // scrapper_evnt.remove_step(camera_idx, 6);
    // scrapper_evnt.remove_step(camera_idx, 5);
    // scrapper_evnt.remove_step(camera_idx, 4);
    // scrapper_evnt.remove_step(camera_idx, 3);
    // scrapper_evnt.remove_step(camera_idx, 2);
    // scrapper_evnt.remove_step(camera_idx, 1);
    // scrapper_evnt.remove_step(camera_idx, 0);
    let director_idx = scrapper_evnt.get_actoridx_for_name("Director").unwrap();
    scrapper_evnt.remove_step(director_idx, 6);
    scrapper_evnt.remove_step(director_idx, 5);
    scrapper_evnt.remove_step(director_idx, 4);
    scrapper_evnt.remove_step(director_idx, 3);
    let starter_idx = scrapper_evnt.get_actoridx_for_name("@starter").unwrap();
    // scrapper_evnt.remove_step(starter_idx, 6);
    scrapper_evnt.remove_step(starter_idx, 5);
    scrapper_evnt.remove_step(starter_idx, 4);
    scrapper_evnt.remove_step(starter_idx, 3);
    scrapper_evnt.remove_step(starter_idx, 2);
    scrapper_evnt.remove_step(starter_idx, 1);
    scrapper_evnt.remove_step(starter_idx, 0);
    write_evnt(&scrapper_evnt, "scrapper_patched.dot").unwrap();
    let written = write_zev(&zev).unwrap();
    let mut file = File::create("testzev.dat").unwrap();
    file.write(&written).unwrap();
    // for evnt in zev {
    //     println!("event: {}", evnt.get_name());
    //     for actor in evnt.get_actors() {
    //         println!("  actor: {}", actor.get_name());
    //     }
    // }
}

fn write_evnt(evnt: &Event, filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    file.write(&evnt.to_dot_file().as_bytes())?;
    Ok(())
}
