#![feature(trait_alias)]

#![allow(incomplete_features)]
#![feature(lazy_type_alias)]

pub mod telemetry;
pub mod server;

use axum::extract::State;

use axum::{body::Bytes, extract::Path, http::StatusCode, response::Html, routing::*};

use chrono::TimeDelta;
use telemetry::analysis::{self,  TimedReading};
use telemetry::parsing::{TelemetryPacket, ParsedTelemetry};
use server::db::DatabaseCtx;

use core::net::SocketAddr;
use serde::Deserialize;
use std::sync::Arc;

type DBState = Arc<DatabaseCtx>;

async fn post_packets(State(analysis_context): State<DBState>, body: Bytes) -> StatusCode {
    let mut packet: TelemetryPacket = unsafe { std::mem::zeroed() };

    unsafe {
        std::ptr::copy_nonoverlapping(
            body.as_ptr(),
            &mut packet as *mut TelemetryPacket as *mut u8,
            body.len(),
        );
    }
    
    match ParsedTelemetry::try_from(&packet) {
        Err(msg) => {
            println!("{}", msg);
            StatusCode::UNPROCESSABLE_ENTITY
        }

        Ok(parsed_telemetry) => {
            analysis_context.submit_readings(&parsed_telemetry); 
            StatusCode::OK 
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum PlotTimeUnit {
    Year,
    Month,
    Day,
    Hour,
}

impl PlotTimeUnit {
    fn to_seconds(&self) -> i64 {
        match self {
            PlotTimeUnit::Year => 31_536_000, 
            PlotTimeUnit::Month => 2_592_000, 
            PlotTimeUnit::Day => 86_400,      
            PlotTimeUnit::Hour => 3_600,     
        }
    }
}

async fn get_timed_ph_plot(
    State(database): State<DBState>,
    Path(time_delta): Path<PlotTimeUnit>,
) -> Html<String> {
    let general_telemetry_since = database.get_readings_since(chrono::Local::now() - TimeDelta::seconds(time_delta.to_seconds()));
    
    let ph_telemetry_since: Vec<TimedReading<f64>> = general_telemetry_since.iter().map(|tel| tel.ph).collect();

    Html(analysis::graphs::plot(&ph_telemetry_since).to_html())
}

async fn get_timed_tds_plot(
    State(database): State<DBState>,
    Path(time_delta): Path<PlotTimeUnit>,
) -> Html<String> {
    let general_telemetry_since = database.get_readings_since(chrono::Local::now() - TimeDelta::seconds(time_delta.to_seconds()));
    
    let tds_telemetry_since: Vec<TimedReading<f64>> = general_telemetry_since.iter().map(|tel| tel.tds).collect();

    Html(analysis::graphs::plot(&tds_telemetry_since).to_html())
}

#[tokio::main]
async fn main() {
    let a_ctx = Arc::new(DatabaseCtx::new("db/readings.sql"));

    let app = Router::new()
        .route("/water", post(post_packets))
        .with_state(a_ctx.clone())
        .route("/ph/:time", get(get_timed_ph_plot))
        .with_state(a_ctx.clone())
        .route("/tds/:time", get(get_timed_tds_plot))
        .with_state(a_ctx.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("Listening on {}", addr);

    // Run the Axum server
    axum_server::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
