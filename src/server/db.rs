use crate::telemetry::parsing::{ParsedTelemetry, TelemetryPacket};
use chrono::Local;

use std::sync::Mutex;

pub type Time = chrono::DateTime<Local>;

pub(crate) struct DatabaseCtx {
    // Rusqlite isn't thread-safe. TODO: Not use Rusqlite.
    readings_db: Mutex<rusqlite::Connection>,

    #[allow(unused)]
    start_time: Time,
}

impl DatabaseCtx {
    pub(crate) fn new(db_name: &str) -> Self {
        Self {
            start_time: Local::now(),
            readings_db: Mutex::new(rusqlite::Connection::open(db_name).unwrap()),
        }
    }
    
    #[allow(unused)]
    pub(crate) fn uptime(&self) -> chrono::TimeDelta {
        Local::now() - self.start_time
    }

    pub fn submit_readings(&self, packet: &ParsedTelemetry) {
        // Yes, it _is_ roundabout of us to store the send times as unix timestamps in the DB.
        // We do it anyway.
        let unix_ph_timestamp = packet.ph.1.timestamp();
        let unix_tds_timestamp = packet.tds.1.timestamp();
        let unix_both_timestamp = packet.time.timestamp();

        self.readings_db
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO readings (pH, pH_timestamp, TDS, TDS_timestamp, both_sent_timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    packet.ph.0,
                    unix_ph_timestamp,
                    packet.tds.0,
                    unix_tds_timestamp,
                    unix_both_timestamp
                ],
            )
            .unwrap();
    }

    pub fn get_readings_since(&self, since: Time) -> Vec<ParsedTelemetry> {
        let timestamp = since.timestamp();

        let db = self.readings_db.lock().unwrap();
        let mut stmt = db.prepare("SELECT pH, pH_timestamp, TDS, TDS_timestamp, both_sent_timestamp FROM readings WHERE both_sent_timestamp > ?1")
                       .unwrap();

        let readings_iter = stmt
            .query_map([timestamp], |row| {
                let ph: f64 = row.get(0).unwrap();
                let ph_timestamp: i64 = row.get::<_, i64>(1).unwrap();

                let tds: f64 = row.get(2).unwrap();
                let tds_timestamp: i64 = row.get::<_, i64>(3).unwrap();

                let time_sent: i64 = row.get::<_, i64>(4).unwrap();

                Ok(TelemetryPacket {
                    ph,
                    ph_timestamp,
                    tds,
                    tds_timestamp,
                    packet_timestamp: time_sent,
                })
            })
            .unwrap();

        let mut readings = Vec::new();
        for reading in readings_iter {
            match reading {
                Ok(raw_readings) => {
                    // If the `.unwrap()` fails here, ie. parsing these readings fails,
                    // it's because we've been pushing junk data to the DB; if that happens (pray that it doesn't)
                    // the problem is much, much deeper than just this function, and it really is better for us to just
                    // crash here than keep inserting junk into the DB.
                    readings.push(ParsedTelemetry::try_from(&raw_readings).expect(
                        "Couldn't parse telemetry data from the database; possible corruption?",
                    ));
                }
                Err(_) => continue,
            }
        }

        readings
    }
}
