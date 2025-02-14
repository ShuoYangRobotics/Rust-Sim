use chrono::Utc;
use nalgebra::DVector;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct Logger {
    file: Arc<Mutex<std::fs::File>>,
    counter: Arc<Mutex<u64>>,
}

impl Logger {
    pub fn new() -> Self {
        let now = Utc::now();
        let filename = format!("least_squares_log_{}.txt", now.format("%Y%m%d_%H%M%S"));

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
