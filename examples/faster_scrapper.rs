use std::env;
use std::fs::File;
use std::io::{Read, Write};

use zeldaevent::zevfile::{parse_zev, write_zev, Event};

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
    let mut file = File::open("../ss-object-map/scripts/sstools/allzev/F300_zev.dat").unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let mut zev = parse_zev(buf.as_slice()).unwrap();
    // println!("{:?}", zev);
    let scrapper_evnt = zev
        .iter_mut()
        .filter(|z| z.get_name() == "SalbageFayCall2")
        .next()
        .unwrap();
    write_evnt(&scrapper_evnt, "scrapper_orig.dot").unwrap();
    // scrapper_evnt.remove_all_waits();
    let scrapper_idx = scrapper_evnt.get_actoridx_for_name("NpcSlrb").unwrap();
    let camera_idx = scrapper_evnt.get_actoridx_for_name("Camera").unwrap();
    let director_idx = scrapper_evnt.get_actoridx_for_name("Director").unwrap();
    let talk_kensei_idx = scrapper_evnt.get_actoridx_for_name("@starter").unwrap();
    let link_idx = scrapper_evnt.get_actoridx_for_name("Link").unwrap();
    // scrapper_evnt.remove_waiting(scrapper_idx, 1);
    // scrapper_evnt.remove_waiting(scrapper_idx, 2);
    // scrapper_evnt.remove_waiting(scrapper_idx, 3);
    // for idx in (2..=9).rev() {
    //     scrapper_evnt.remove_step(scrapper_idx, idx);
    // }
    // for idx in (1..=6).rev() {
    //     scrapper_evnt.remove_step(director_idx, idx);
    // }
    // for idx in (1..=2).rev() {
    //     scrapper_evnt.remove_step(camera_idx, idx);
    // }
    // for idx in (1..=5).rev() {
    //     if idx == 2 {
    //         continue;
    //     }
    //     scrapper_evnt.remove_step(talk_kensei_idx, idx);
    // }
    // scrapper_evnt
    //     .add_wait(
    //         director_idx,
    //         idx_by_name(&scrapper_evnt, director_idx, "FadeOut"),
    //         talk_kensei_idx,
    //         idx_by_name(&scrapper_evnt, talk_kensei_idx, "Kira"),
    //     )
    //     .unwrap();
    scrapper_evnt.remove_step(scrapper_idx, 2);
    scrapper_evnt.remove_step(scrapper_idx, 1);
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
