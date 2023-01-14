use std::collections::{HashMap, HashSet};

use zeldaevent::zevfile::{parse_zev, write_zev};

// FastTravelAmiiboReturn
// FastTravelAmiibo

pub fn main() {
    let data_hd = std::fs::read("../common-exp/common_hd_zev.dat").unwrap();
    let data_sd = std::fs::read("../common-exp/common_sd_zev.dat").unwrap();
    let hd_events = parse_zev(&data_hd).unwrap();
    let mut sd_events = parse_zev(&data_sd).unwrap();
    let amiibo_return = hd_events
        .iter()
        .find(|e| e.name == "FastTravelAmiiboReturn")
        .unwrap();
    let amiibo = hd_events
        .iter()
        .find(|e| e.name == "FastTravelAmiibo")
        .unwrap();
    sd_events.push(amiibo.clone());
    sd_events.push(amiibo_return.clone());
    let patched_sd_events = write_zev(&sd_events).unwrap();
    std::fs::write("../common-exp/common_sd_patched_zev.dat", patched_sd_events).unwrap();
}
