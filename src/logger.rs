use chrono::Utc;
use nalgebra::DVector;
use plotters::prelude::*;
use serde_json::json;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tempfile::tempfile;

pub struct Logger {
    file: Arc<Mutex<std::fs::File>>,
    counter: Arc<Mutex<u64>>,
}

impl Logger {
    pub fn new() -> Self {
        let now = Utc::now();
        let filename = format!("log/least_squares_log_{}.txt", now.format("%Y%m%d_%H%M%S"));

        fs::create_dir_all("log").expect("Unable to create log directory");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filename)
            .expect("Unable to open log file");

        Logger {
            file: Arc::new(Mutex::new(file)),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn new_for_analysis() -> Self {
        let temp_file = tempfile().expect("Unable to create temporary file");

        Logger {
            file: Arc::new(Mutex::new(temp_file)),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn log(&self, dimension: usize, solution: &DVector<f64>, time_taken_us: u128) {
        let mut file = self.file.lock().unwrap();
        let mut counter = self.counter.lock().unwrap();

        *counter += 1;

        let log_entry = format!(
            "{}\t{}\t{}\t{}\n",
            *counter,
            dimension,
            json!(solution.as_slice()),
            time_taken_us
        );

        file.write_all(log_entry.as_bytes())
            .expect("Unable to write to log file");
        file.sync_all().expect("Unable to sync log file");
    }

    pub fn analyze(&self, filename: &str) {
        let file = File::open(filename).expect("Unable to open log file");
        let reader = BufReader::new(file);

        let mut counters = Vec::new();
        let mut times = Vec::new();

        for line in reader.lines() {
            let line = line.expect("Unable to read line");
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 4 {
                let counter: u64 = parts[0].parse().expect("Unable to parse counter");
                let time_taken_us: u128 = parts[3].parse().expect("Unable to parse time taken");

                counters.push(counter);
                times.push(time_taken_us);
            }
        }

        // Plot the data
        fs::create_dir_all("log/analyze").expect("Unable to create analyze directory");
        let plot_filename = format!(
            "log/analyze/plot_{}.png",
            filename.replace("log/", "").replace(".txt", "")
        );
        let root = BitMapBackend::new(&plot_filename, (640, 480)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("Time Taken vs Counter", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(
                counters[0]..counters[counters.len() - 1],
                0..*times.iter().max().unwrap() as u32,
            )
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(
                counters
                    .iter()
                    .zip(times.iter())
                    .map(|(&x, &y)| (x, y as u32)),
                &RED,
            ))
            .unwrap()
            .label("Time Taken")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE)
            .draw()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_format() {
        let logger = Logger::new();
        let solution = DVector::from_row_slice(&[1.0, 2.0, 3.0]);
        let time_taken_us = 123456;

        logger.log(3, &solution, time_taken_us);

        let log_entry = format!(
            "1\t3\t{}\t{}\n",
            serde_json::json!(solution.as_slice()),
            time_taken_us
        );

        assert!(is_valid_log_entry(&log_entry));
    }

    fn is_valid_log_entry(entry: &str) -> bool {
        let parts: Vec<&str> = entry.trim().split('\t').collect();
        if parts.len() != 4 {
            return false;
        }

        if parts[0].parse::<u64>().is_err() {
            return false;
        }

        let dimension: usize = match parts[1].parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        let solution: Vec<f64> = match serde_json::from_str(parts[2]) {
            Ok(s) => s,
            Err(_) => return false,
        };

        if solution.len() != dimension {
            return false;
        }

        if parts[3].parse::<u128>().is_err() {
            return false;
        }

        true
    }
}
