use plotly::{Plot, Scatter};

use num_traits::Float;

// We can change this later once we know more about what the sensors throw at us.
pub(crate) trait Reading = Float;

pub(crate) type ReadingTime = chrono::DateTime<chrono::Local>;
pub(crate) type TimedReading<T: Reading> = (T, ReadingTime);


pub(crate) mod graphs 
{
    use crate::telemetry::analysis::*;
 
    pub(crate) fn plot(data: &[TimedReading<f64>]) -> Plot {
        let readings = data;

        let values_and_times: (Vec<f64>, Vec<ReadingTime>) =
            readings.iter().map(|read| (read.0, read.1)).unzip();

        let mut plot = Plot::new();

        let trace = Scatter::new(
            values_and_times
                .1
                .iter()
                .map(|time| time.to_rfc2822())
                .collect(),
            values_and_times.0,
        );

        plot.add_trace(trace);
        plot
    }
}
