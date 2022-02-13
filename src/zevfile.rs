use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::cmp::Ordering;
use std::io::{Cursor, Read, Write};

use crate::raw::{RawActor, RawDataDef, RawEvent, RawHeader, RawStep1, RawStep2};

const MAGIC: u16 = 0x775A; // "wZ"
const EV: u16 = 0x4576; // "Ev"

const STEP1_SIZE: usize = 0x1C;
const STEP2_SIZE: usize = 0xC;
const DATA_DEF_SIZE: usize = 0xC;
const INT_SIZE: usize = 4;
const FLOAT_SIZE: usize = 4;

#[derive(Debug)]
pub struct Event {
    name: String,
    pub unk1: u8,

    actors: Vec<Actor>,
    wait_fors: Vec<WaitFor>,
}

#[derive(Debug)]
pub struct Actor {
    name: String,
    pub unk1: u16,
    pub unk2: u16,

    steps: Vec<Step>,
}

#[derive(Debug)]
pub struct Step {
    // part1
    long_name: String,
    // wait_for
    actor_index: u16,
    unk1: u16,
    // dummy0
    // thisindex
    // dummy1
    // part2
    name: String,
    unk2: u16,
    // thisindex
    data: Vec<StepData>,
}

#[derive(Debug)]
pub struct StepData {
    name: String,
    unk1: u16,
    values: StepDataValues,
}

#[derive(Debug)]
pub enum StepDataValues {
    Ints(Vec<u32>),
    Floats(Vec<f32>),
    String(String),
}

#[derive(Debug)]
pub enum ZevParseError {
    InvalidHeader(String),
    InvalidFile(String),
}

impl From<std::io::Error> for ZevParseError {
    fn from(e: std::io::Error) -> Self {
        ZevParseError::InvalidFile(format!("unexpected io error: {:?}", e))
    }
}

#[derive(Debug)]
pub enum ZevWriteError {
    LogicError(String),
}

impl From<std::io::Error> for ZevWriteError {
    fn from(e: std::io::Error) -> Self {
        ZevWriteError::LogicError(format!("unexpected io error: {:?}", e))
    }
}

#[derive(Debug)]
pub enum MutationError {
    StringNotAscii,
    StringTooLong,
}

#[derive(Debug)]
struct WaitFor {
    waiting: StepRef,
    waiting_on: StepRef,
}

#[derive(Debug)]
struct StepRef {
    actor_idx: u16,
    step_idx: u16,
}

impl Event {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) -> Result<(), MutationError> {
        if !name.as_bytes().is_ascii() {
            return Err(MutationError::StringNotAscii);
        }
        if name.as_bytes().len() > 32 {
            return Err(MutationError::StringTooLong);
        }
        self.name = name;
        Ok(())
    }

    pub fn get_actors(&self) -> &Vec<Actor> {
        &self.actors
    }
}

impl Actor {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) -> Result<(), MutationError> {
        if !name.as_bytes().is_ascii() {
            return Err(MutationError::StringNotAscii);
        }
        if name.as_bytes().len() > 32 {
            return Err(MutationError::StringTooLong);
        }
        self.name = name;
        Ok(())
    }
}

fn nin_sort(s1: &String, s2: &String) -> Ordering {
    for (c1, c2) in s1
        .bytes()
        .chain(std::iter::once(0))
        .zip(s2.bytes().chain(std::iter::once(0)))
    {
        let cmp = c1.cmp(&c2);
        if cmp != Ordering::Equal {
            return cmp;
        }
    }
    return Ordering::Equal;
}

pub fn parse_zev(bytes: &[u8]) -> Result<Vec<Event>, ZevParseError> {
    let mut c = Cursor::new(bytes);
    let header = RawHeader::read(&mut c)?;
    if header.magic != MAGIC {
        return Err(ZevParseError::InvalidHeader(format!(
            "Wrong magic, expected {:#} got {:#}",
            MAGIC, header.magic
        )));
    }
    if header.stepscount != header.steps2count {
        return Err(ZevParseError::InvalidHeader(format!(
            "steps1 and steps2 don't have the same count: {} != {}",
            header.stepscount, header.steps2count
        )));
    }
    if header.alwaysev != EV {
        return Err(ZevParseError::InvalidHeader(format!(
            "expected {:#} got {:#}",
            EV, header.alwaysev
        )));
    }
    let event_offset = RawHeader::SIZE;
    let actor_offset = event_offset + header.evntcount as usize * RawEvent::SIZE;
    let step1_offset = actor_offset + header.actorcount as usize * RawActor::SIZE;
    let step2_offset = step1_offset + header.stepscount as usize * STEP1_SIZE;
    let data_def_offset = step2_offset + header.stepscount as usize * STEP2_SIZE;
    let ints_offset = data_def_offset + header.datacount as usize * DATA_DEF_SIZE;
    let floats_offset = ints_offset + header.intcount as usize * INT_SIZE;
    let strings_offset = floats_offset + header.floatcount as usize * FLOAT_SIZE;
    let expected_len = strings_offset + header.stringcount as usize;
    if expected_len != bytes.len() {
        return Err(ZevParseError::InvalidFile(format!(
            "expected file len of {}, got {}",
            expected_len,
            bytes.len()
        )));
    }
    let mut raw_events = Vec::new();
    for evntidx in 0..header.evntcount {
        c.set_position((event_offset + evntidx as usize * RawEvent::SIZE) as u64);
        raw_events.push(RawEvent::read(&mut c)?);
    }
    // out events are sorted by name, but not the other event stuff...
    raw_events.sort_by_key(|e| e.actorindex);
    let mut events = Vec::new();
    for raw_event in raw_events {
        let ac_start = raw_event.actorindex as usize;
        let ac_end = ac_start + raw_event.actorcount as usize;

        let mut actors = Vec::new();
        let mut wait_fors = Vec::new();
        for actoridx in ac_start..ac_end {
            c.set_position((actor_offset + actoridx as usize * RawActor::SIZE) as u64);
            let raw_actor = RawActor::read(&mut c)?;

            let step_start = raw_actor.stepindex as usize;
            let step_end = step_start + raw_actor.stepcount as usize;

            let mut steps = Vec::new();

            for stepidx in step_start..step_end {
                c.set_position((step1_offset + stepidx as usize * RawStep1::SIZE) as u64);
                let step1 = RawStep1::read(&mut c)?;
                c.set_position((step2_offset + stepidx as usize * RawStep2::SIZE) as u64);
                let step2 = RawStep2::read(&mut c)?;

                let data_def_start = step2.datadefindex as usize;
                let data_def_end = data_def_start + step2.datadefcount as usize;

                let mut stepdatas = Vec::new();

                for data_def_idx in data_def_start..data_def_end {
                    c.set_position(
                        (data_def_offset + data_def_idx as usize * RawDataDef::SIZE) as u64,
                    );
                    let data_def = RawDataDef::read(&mut c)?;

                    let values = match data_def.datatype {
                        0 => {
                            let mut values = Vec::new();
                            // ints
                            c.set_position(
                                (ints_offset + data_def.dataindex as usize * INT_SIZE) as u64,
                            );
                            for _ in 0..data_def.datalen {
                                values.push(c.read_u32::<BE>()?);
                            }
                            StepDataValues::Ints(values)
                        }
                        1 => {
                            let mut values = Vec::new();
                            // floats
                            c.set_position(
                                (floats_offset + data_def.dataindex as usize * FLOAT_SIZE) as u64,
                            );
                            for _ in 0..data_def.datalen {
                                values.push(c.read_f32::<BE>()?);
                            }
                            StepDataValues::Floats(values)
                        }
                        2 => {
                            // String
                            c.set_position((strings_offset + data_def.dataindex as usize) as u64);
                            let mut buf = vec![0; data_def.datalen as usize];
                            c.read_exact(&mut buf)?;
                            // zero terminated
                            if buf.pop().unwrap_or(0) != 0 {
                                return Err(ZevParseError::InvalidFile(format!(
                                    "error string value not null terminated: {:?}",
                                    buf
                                )));
                            }
                            StepDataValues::String(String::from_utf8(buf).map_err(|e| {
                                ZevParseError::InvalidFile(format!(
                                    "error parsing string value: {:?}",
                                    e.as_bytes()
                                ))
                            })?)
                        }
                        _ => {
                            return Err(ZevParseError::InvalidFile(format!(
                                "invalid datatype: {}",
                                data_def.datatype
                            )))
                        }
                    };

                    stepdatas.push(StepData {
                        name: data_def.name,
                        unk1: data_def.unk1,
                        values,
                    });
                }

                // basically checking if waitfor is positive, but converting to a
                // positive only number at the same time
                if let Ok(wait_for) = u16::try_from(step1.waitfor) {
                    // TODO: wait for
                    let waiting_actor_idx = actoridx as u16 - raw_event.actorindex;
                    let waiting_step_idx = stepidx as u16 - raw_actor.stepindex;

                    // read other step
                    c.set_position((step1_offset + wait_for as usize * RawStep1::SIZE) as u64);
                    let waited_for_step = RawStep1::read(&mut c)?;

                    c.set_position(
                        (actor_offset + waited_for_step.actorindex as usize * RawActor::SIZE)
                            as u64,
                    );
                    let waited_for_actor = RawActor::read(&mut c)?;

                    let waited_for_actor_idx = waited_for_step.actorindex - raw_event.actorindex;
                    let waited_for_step_idx = wait_for - waited_for_actor.stepindex;

                    wait_fors.push(WaitFor {
                        waiting: StepRef {
                            actor_idx: waiting_actor_idx,
                            step_idx: waiting_step_idx,
                        },
                        waiting_on: StepRef {
                            actor_idx: waited_for_actor_idx,
                            step_idx: waited_for_step_idx,
                        },
                    });
                }

                steps.push(Step {
                    long_name: step1.name,
                    name: step2.name,
                    actor_index: step1.actorindex,
                    unk1: step1.unk3,
                    unk2: step2.unk1,
                    data: stepdatas,
                });
            }

            actors.push(Actor {
                name: raw_actor.name,
                unk1: raw_actor.unk1,
                unk2: raw_actor.unk2,
                steps,
            });
        }

        events.push(Event {
            name: raw_event.name,
            unk1: raw_event.unk1,
            actors,
            wait_fors,
        });
    }
    Ok(events)
}

pub fn write_zev(zevs: &Vec<Event>) -> Result<Vec<u8>, ZevWriteError> {
    // first, we sum up all the counts
    let mut evntcount = 0;
    let mut actorcount = 0;
    let mut stepscount = 0;
    let mut datadefcount = 0;
    let mut intcount = 0;
    let mut floatcount = 0;
    let mut stringcount = 0;

    for evnt in zevs.iter() {
        evntcount += 1;
        for actor in evnt.actors.iter() {
            actorcount += 1;
            for step in actor.steps.iter() {
                stepscount += 1;
                for datadef in step.data.iter() {
                    datadefcount += 1;
                    match &datadef.values {
                        StepDataValues::Ints(ints) => intcount += ints.len(),
                        StepDataValues::Floats(floats) => floatcount += floats.len(),
                        StepDataValues::String(string) => stringcount += string.len() + 1,
                    };
                }
            }
        }
    }

    let event_offset = RawHeader::SIZE;
    let actor_offset = event_offset + evntcount as usize * RawEvent::SIZE;
    let step1_offset = actor_offset + actorcount as usize * RawActor::SIZE;
    let step2_offset = step1_offset + stepscount as usize * STEP1_SIZE;
    let data_def_offset = step2_offset + stepscount as usize * STEP2_SIZE;
    let ints_offset = data_def_offset + datadefcount as usize * DATA_DEF_SIZE;
    let floats_offset = ints_offset + intcount as usize * INT_SIZE;
    let strings_offset = floats_offset + floatcount as usize * FLOAT_SIZE;
    let expected_len = strings_offset + stringcount as usize;

    let header = RawHeader {
        magic: MAGIC,
        evntcount,
        actorcount,
        stepscount,
        steps2count: stepscount,
        alwaysev: EV,
        datacount: datadefcount,
        intcount: intcount as u16,
        floatcount: floatcount as u16,
        stringcount: stringcount as u16,
    };

    let mut cur_actor_idx = 0;
    let mut cur_step_idx = 0;
    let mut cur_datadef_idx = 0;
    let mut cur_int_idx = 0;
    let mut cur_float_idx = 0;
    let mut cur_string_idx = 0;

    let mut c = Cursor::new(Vec::with_capacity(expected_len));

    header.write(&mut c)?;

    let mut raw_evnts = Vec::with_capacity(zevs.len());

    for evnt in zevs.iter() {
        let raw_evnt = RawEvent {
            name: evnt.name.clone(),
            dummy1: 0,
            dummy2: 0,
            unk1: evnt.unk1,
            actorindex: cur_actor_idx,
            actorcount: evnt.actors.len() as u16,
        };
        // we need to sort them later
        raw_evnts.push(raw_evnt);

        let mut actor_step_offsets = Vec::with_capacity(evnt.actors.len());
        for actor in evnt.actors.iter() {
            actor_step_offsets.push(cur_step_idx);

            c.set_position((actor_offset + cur_actor_idx as usize * RawActor::SIZE) as u64);

            RawActor {
                name: actor.name.clone(),
                unk1: actor.unk1,
                unk2: actor.unk2,
                stepindex: cur_step_idx,
                stepcount: actor.steps.len() as u16,
            }
            .write(&mut c)?;

            for step in actor.steps.iter() {
                c.set_position((step1_offset + cur_step_idx as usize * RawStep1::SIZE) as u64);

                RawStep1 {
                    name: step.long_name.clone(),
                    waitfor: -1, // will be filled later
                    unk3: step.unk1,
                    actorindex: cur_actor_idx,
                    dummy0: 0,
                    dummy1: 1,
                    thisindex: cur_step_idx,
                }
                .write(&mut c)?;

                c.set_position((step2_offset + cur_step_idx as usize * RawStep2::SIZE) as u64);

                RawStep2 {
                    name: step.name.clone(),
                    unk1: step.unk2,
                    thisindex: cur_step_idx,
                    datadefindex: cur_datadef_idx,
                    datadefcount: step.data.len() as u16,
                }
                .write(&mut c)?;

                cur_step_idx += 1;

                for datadef in step.data.iter() {
                    let (typ, idx, len) = match &datadef.values {
                        StepDataValues::Ints(ints) => {
                            let idx = cur_int_idx;
                            c.set_position((ints_offset + cur_int_idx as usize * INT_SIZE) as u64);
                            for int in ints.iter() {
                                c.write_u32::<BE>(*int)?;
                            }
                            cur_int_idx += ints.len();
                            (0, idx, ints.len())
                        }
                        StepDataValues::Floats(floats) => {
                            let idx = cur_float_idx;
                            c.set_position(
                                (floats_offset + cur_float_idx as usize * FLOAT_SIZE) as u64,
                            );
                            for float in floats.iter() {
                                c.write_f32::<BE>(*float)?;
                            }
                            cur_float_idx += floats.len();
                            (1, idx, floats.len())
                        }
                        StepDataValues::String(string) => {
                            let idx = cur_string_idx;
                            c.set_position((strings_offset + cur_string_idx as usize) as u64);
                            c.write_all(string.as_bytes())?;
                            c.write_u8(0)?; // null teminated
                            cur_string_idx += string.len() + 1;
                            (2, idx, string.len() + 1)
                        }
                    };

                    c.set_position(
                        (data_def_offset + cur_datadef_idx as usize * RawDataDef::SIZE) as u64,
                    );

                    RawDataDef {
                        name: datadef.name.clone(),
                        unk1: datadef.unk1,
                        datatype: typ,
                        dataindex: idx as u16,
                        datalen: len as u16,
                    }
                    .write(&mut c)?;

                    cur_datadef_idx += 1;
                }
            }

            cur_actor_idx += 1;
        }

        for WaitFor {
            waiting,
            waiting_on,
        } in evnt.wait_fors.iter()
        {
            let waiting_idx =
                actor_step_offsets.get(waiting.actor_idx as usize).unwrap() + waiting.step_idx;
            let waiting_on_idx = actor_step_offsets
                .get(waiting_on.actor_idx as usize)
                .unwrap()
                + waiting_on.step_idx;

            // wait_for is after name
            c.set_position((step1_offset + waiting_idx as usize * RawStep1::SIZE + 0x10) as u64);
            c.write_u16::<BE>(waiting_on_idx)?;
        }
    }

    raw_evnts.sort_by(|a, b| nin_sort(&a.name, &b.name));

    c.set_position(event_offset as u64);
    for raw_evnt in raw_evnts {
        raw_evnt.write(&mut c)?
    }

    Ok(c.into_inner())
}
