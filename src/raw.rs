use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::io::{Read, Write};

use crate::zevfile::ZevParseError;

fn read_null_term_string<R: Read>(r: &mut R, max_len: usize) -> Result<String, ZevParseError> {
    let mut buf = vec![0; max_len];
    r.read_exact(&mut buf)?;
    // maybe there isn't actually a null at the end, then take the entire byte array
    if let Some(end) = buf.iter().position(|c| *c == 0) {
        String::from_utf8(buf[0..end].to_vec())
    } else {
        String::from_utf8(buf)
    }
    .map_err(|_| ZevParseError::InvalidFile(format!("invalid string")))
}

fn write_null_term_pad_string<W: Write>(
    w: &mut W,
    s: &String,
    max_len: usize,
) -> Result<(), std::io::Error> {
    let as_bytes = s.as_bytes();
    if as_bytes.len() > max_len {
        // TODO, probably change to error
        // and not panic
        panic!("string too long!");
    }
    w.write_all(as_bytes)?;
    let pad_left = max_len.checked_sub(as_bytes.len()).unwrap_or(0);
    for _ in 0..pad_left {
        w.write_u8(0)?;
    }
    Ok(())
}

#[derive(Debug)]
pub(crate) struct RawHeader {
    pub(crate) magic: u16,
    pub(crate) evntcount: u16,
    pub(crate) actorcount: u16,
    pub(crate) stepscount: u16,
    pub(crate) steps2count: u16,
    pub(crate) datacount: u16,
    pub(crate) alwaysev: u16,
    pub(crate) intcount: u16,
    pub(crate) floatcount: u16,
    pub(crate) stringcount: u16,
}

impl RawHeader {
    pub const SIZE: usize = 0x14;

    pub fn read<R: Read>(r: &mut R) -> Result<Self, ZevParseError> {
        Ok(RawHeader {
            magic: r.read_u16::<BE>()?,
            evntcount: r.read_u16::<BE>()?,
            actorcount: r.read_u16::<BE>()?,
            stepscount: r.read_u16::<BE>()?,
            steps2count: r.read_u16::<BE>()?,
            datacount: r.read_u16::<BE>()?,
            alwaysev: r.read_u16::<BE>()?,
            intcount: r.read_u16::<BE>()?,
            floatcount: r.read_u16::<BE>()?,
            stringcount: r.read_u16::<BE>()?,
        })
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        w.write_u16::<BE>(self.magic)?;
        w.write_u16::<BE>(self.evntcount)?;
        w.write_u16::<BE>(self.actorcount)?;
        w.write_u16::<BE>(self.stepscount)?;
        w.write_u16::<BE>(self.steps2count)?;
        w.write_u16::<BE>(self.datacount)?;
        w.write_u16::<BE>(self.alwaysev)?;
        w.write_u16::<BE>(self.intcount)?;
        w.write_u16::<BE>(self.floatcount)?;
        w.write_u16::<BE>(self.stringcount)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct RawEvent {
    pub(crate) name: String,
    pub(crate) dummy1: u8,
    pub(crate) unk1: u8,
    pub(crate) dummy2: u16,
    pub(crate) actorindex: u16,
    pub(crate) actorcount: u16,
}

impl RawEvent {
    pub const SIZE: usize = 0x28;

    pub fn read<R: Read>(r: &mut R) -> Result<Self, ZevParseError> {
        let name = read_null_term_string(r, 0x20)?;
        Ok(RawEvent {
            name,
            dummy1: r.read_u8()?,
            unk1: r.read_u8()?,
            dummy2: r.read_u16::<BE>()?,
            actorindex: r.read_u16::<BE>()?,
            actorcount: r.read_u16::<BE>()?,
        })
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        write_null_term_pad_string(w, &self.name, 0x20)?;
        w.write_u8(self.dummy1)?;
        w.write_u8(self.unk1)?;
        w.write_u16::<BE>(self.dummy2)?;
        w.write_u16::<BE>(self.actorindex)?;
        w.write_u16::<BE>(self.actorcount)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct RawActor {
    pub(crate) name: String,
    pub(crate) unk1: u16,
    pub(crate) unk2: u16,
    pub(crate) stepindex: u16,
    pub(crate) stepcount: u16,
}

impl RawActor {
    pub const SIZE: usize = 0x28;

    pub fn read<R: Read>(r: &mut R) -> Result<Self, ZevParseError> {
        let name = read_null_term_string(r, 0x20)?;
        Ok(RawActor {
            name,
            unk1: r.read_u16::<BE>()?,
            unk2: r.read_u16::<BE>()?,
            stepindex: r.read_u16::<BE>()?,
            stepcount: r.read_u16::<BE>()?,
        })
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        write_null_term_pad_string(w, &self.name, 0x20)?;
        w.write_u16::<BE>(self.unk1)?;
        w.write_u16::<BE>(self.unk2)?;
        w.write_u16::<BE>(self.stepindex)?;
        w.write_u16::<BE>(self.stepcount)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct RawStep1 {
    pub(crate) name: String,
    pub(crate) waitfor: i16,
    pub(crate) actorindex: u16,
    pub(crate) unk3: u16,
    pub(crate) dummy0: u16,
    pub(crate) thisindex: u16,
    pub(crate) dummy1: u16,
}

impl RawStep1 {
    pub const SIZE: usize = 0x1C;

    pub fn read<R: Read>(r: &mut R) -> Result<Self, ZevParseError> {
        let name = read_null_term_string(r, 0x10)?;
        Ok(RawStep1 {
            name,
            waitfor: r.read_i16::<BE>()?,
            actorindex: r.read_u16::<BE>()?,
            unk3: r.read_u16::<BE>()?,
            dummy0: r.read_u16::<BE>()?,
            thisindex: r.read_u16::<BE>()?,
            dummy1: r.read_u16::<BE>()?,
        })
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        write_null_term_pad_string(w, &self.name, 0x10)?;
        w.write_i16::<BE>(self.waitfor)?;
        w.write_u16::<BE>(self.actorindex)?;
        w.write_u16::<BE>(self.unk3)?;
        w.write_u16::<BE>(self.dummy0)?;
        w.write_u16::<BE>(self.thisindex)?;
        w.write_u16::<BE>(self.dummy1)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct RawStep2 {
    pub(crate) name: String,
    pub(crate) unk1: u16,
    pub(crate) thisindex: u16,
    pub(crate) datadefindex: u16,
    pub(crate) datadefcount: u16,
}

impl RawStep2 {
    pub const SIZE: usize = 0xC;

    pub fn read<R: Read>(r: &mut R) -> Result<Self, ZevParseError> {
        let name = read_null_term_string(r, 4)?;
        Ok(RawStep2 {
            name,
            unk1: r.read_u16::<BE>()?,
            thisindex: r.read_u16::<BE>()?,
            datadefindex: r.read_u16::<BE>()?,
            datadefcount: r.read_u16::<BE>()?,
        })
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        write_null_term_pad_string(w, &self.name, 4)?;
        w.write_u16::<BE>(self.unk1)?;
        w.write_u16::<BE>(self.thisindex)?;
        w.write_u16::<BE>(self.datadefindex)?;
        w.write_u16::<BE>(self.datadefcount)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct RawDataDef {
    pub(crate) name: String,
    pub(crate) unk1: u16,
    pub(crate) datatype: u16,
    pub(crate) dataindex: u16,
    pub(crate) datalen: u16,
}

impl RawDataDef {
    pub const SIZE: usize = 0xC;

    pub fn read<R: Read>(r: &mut R) -> Result<Self, ZevParseError> {
        let name = read_null_term_string(r, 4)?;
        Ok(RawDataDef {
            name,
            unk1: r.read_u16::<BE>()?,
            datatype: r.read_u16::<BE>()?,
            dataindex: r.read_u16::<BE>()?,
            datalen: r.read_u16::<BE>()?,
        })
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), std::io::Error> {
        write_null_term_pad_string(w, &self.name, 4)?;
        w.write_u16::<BE>(self.unk1)?;
        w.write_u16::<BE>(self.datatype)?;
        w.write_u16::<BE>(self.dataindex)?;
        w.write_u16::<BE>(self.datalen)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::{RawActor, RawEvent, RawHeader, RawStep1, RawStep2};
    use std::io::Cursor;
    #[test]
    fn test_raw_header() {
        let bytes = b"wZ\x00\x1d\x00i\x01Z\x01Z\x01\xefEv\x00\xf3\x027\x00\xf7";
        assert_eq!(RawHeader::SIZE, bytes.len());
        let header = RawHeader::read(&mut Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        header.write(&mut out).unwrap();
        assert_eq!(bytes, out.as_slice());
    }

    #[test]
    fn test_raw_event() {
        let bytes = b"BackstairsGossip\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00R\x00\x03";
        assert_eq!(RawEvent::SIZE, bytes.len());
        let event = RawEvent::read(&mut Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        event.write(&mut out).unwrap();
        assert_eq!(bytes, out.as_slice());
        assert_eq!(event.name, "BackstairsGossip");
    }

    #[test]
    fn test_raw_actor() {
        let bytes = b"@player\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x16\x01\x0c\x00\x01";
        assert_eq!(RawActor::SIZE, bytes.len());
        let actor = RawActor::read(&mut Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        actor.write(&mut out).unwrap();
        assert_eq!(bytes, out.as_slice());
        assert_eq!(actor.name, "@player");
    }

    #[test]
    fn test_raw_step1() {
        let bytes = b"Cast\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00B\x00\x00\x00\x00\x00\xcd\x00\x01";
        assert_eq!(RawStep1::SIZE, bytes.len());
        let step = RawStep1::read(&mut Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        step.write(&mut out).unwrap();
        assert_eq!(bytes, out.as_slice());
        assert_eq!(step.name, "Cast");
    }

    #[test]
    fn test_raw_step2() {
        let bytes = b"cast\x00\x05\x00\xcd\x01\x17\x00\x01";
        assert_eq!(RawStep2::SIZE, bytes.len());
        let step = RawStep2::read(&mut Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        step.write(&mut out).unwrap();
        assert_eq!(bytes, out.as_slice());
        assert_eq!(step.name, "cast");
    }
}
