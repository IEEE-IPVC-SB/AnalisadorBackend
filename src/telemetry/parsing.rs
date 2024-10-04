use crate::telemetry::analysis::*;
use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};

///  The packets sent by the mc are of the form:
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(C, packed)]
pub(crate) struct TelemetryPacket {
    pub ph: f64,
    pub ph_timestamp: i64,

    pub tds: f64,
    pub tds_timestamp: i64,

    pub packet_timestamp: i64,
}

// Do note the presence of the last timestamp: as a packet must contain both a pH reading and a TDS reading
// it's useful to know when the packet was actually fully-formed & sent.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ParsedTelemetry {
    pub ph: TimedReading<f64>,
    pub tds: TimedReading<f64>,

    pub time: ReadingTime,
}

impl TryFrom<&TelemetryPacket> for ParsedTelemetry {
    type Error = &'static str;

    fn try_from(raw_packet: &TelemetryPacket) -> Result<Self, Self::Error> {
        let parsed_time_ph = Local.timestamp_opt(raw_packet.ph_timestamp, 0).into();
        let parsed_time_tds = Local.timestamp_opt(raw_packet.tds_timestamp, 0).into();
        let parsed_time_sent = Local.timestamp_opt(raw_packet.packet_timestamp, 0).into();

        match (parsed_time_ph, parsed_time_tds, parsed_time_sent) {
            // Bad packet
            (None, _, _) => Err("Invalid pH timestamp in raw packet."),
            (_, None, _) => Err("Invalid TDS timestamp in raw packet."),
            (_, _, None) => Err("Invalid packet timestamp."),

            // Good packet
            (ph_time, tds_time, pak_time) => {
                let parsed_ph = (raw_packet.ph, ph_time.unwrap().earliest().unwrap());
                let parsed_tds = (raw_packet.tds, tds_time.unwrap().earliest().unwrap());

                Ok(Self {
                    ph: parsed_ph,
                    tds: parsed_tds,
                    time: pak_time.unwrap().earliest().unwrap(),
                })
            }
        }
    }
}
