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
        use plotters::prelude::*;
        use serde_json::Value;
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(filename).expect("Unable to open log file");
        let reader = BufReader::new(file);

        let mut counters = Vec::new();
        let mut times = Vec::new();
        let mut ball1_x = Vec::new();
        let mut ball1_y = Vec::new();
        let mut ball2_x = Vec::new();
        let mut ball2_y = Vec::new();

        for line in reader.lines() {
            let line = line.expect("Unable to read line");
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 4 {
                let counter: u64 = parts[0].parse().expect("Unable to parse counter");
                let time_taken_us: u128 = parts[3].parse().expect("Unable to parse time taken");
                counters.push(counter);
                times.push(time_taken_us);

                // Parse solution JSON
                let json_val: Value = serde_json::from_str(parts[2]).expect("Unable to parse JSON");
                if let Some(arr) = json_val.as_array() {
                    if arr.len() == 4 {
                        ball1_x.push(arr[0].as_f64().expect("Invalid ball1.x"));
                        ball1_y.push(arr[1].as_f64().expect("Invalid ball1.y"));
                        ball2_x.push(arr[2].as_f64().expect("Invalid ball2.x"));
                        ball2_y.push(arr[3].as_f64().expect("Invalid ball2.y"));
                    }
                }
            }
        }

        // Plot time taken vs counter
        fs::create_dir_all("log/analyze").expect("Unable to create analyze directory");
        let plot_time_filename = format!(
            "log/analyze/plot_time_{}.png",
            filename.replace("log/", "").replace(".txt", "")
        );
        let root_time = BitMapBackend::new(&plot_time_filename, (640, 480)).into_drawing_area();
        root_time.fill(&WHITE).unwrap();
        let max_time = *times.iter().max().unwrap_or(&0) as u32;
        let mut chart_time = ChartBuilder::on(&root_time)
            .caption("Time Taken vs Counter", ("sans-serif", 30).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(counters[0]..counters[counters.len() - 1], 0..max_time)
            .unwrap();
        chart_time.configure_mesh().draw().unwrap();
        chart_time
            .draw_series(LineSeries::new(
                counters
                    .iter()
                    .zip(times.iter())
                    .map(|(&x, &y)| (x, y as u32)),
                &BLUE,
            ))
            .unwrap();

        // Plot trajectory for Ball 1
        let plot_ball1_filename = format!(
            "log/analyze/plot_trajectory_ball1_{}.png",
            filename.replace("log/", "").replace(".txt", "")
        );
        let mut root_ball1 =
            BitMapBackend::new(&plot_ball1_filename, (640, 480)).into_drawing_area();
        root_ball1.fill(&WHITE).unwrap();
        let (min_x1, max_x1) = (
            *ball1_x
                .iter()
                .fold(&f64::INFINITY, |a, b| if b < a { b } else { a }),
            *ball1_x
                .iter()
                .fold(&f64::NEG_INFINITY, |a, b| if b > a { b } else { a }),
        );
        let (min_y1, max_y1) = (
            *ball1_y
                .iter()
                .fold(&f64::INFINITY, |a, b| if b < a { b } else { a }),
            *ball1_y
                .iter()
                .fold(&f64::NEG_INFINITY, |a, b| if b > a { b } else { a }),
        );
        let mut chart_ball1 = ChartBuilder::on(&root_ball1)
            .caption("Ball 1 Trajectory", ("sans-serif", 30).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(min_x1..max_x1, min_y1..max_y1)
            .unwrap();
        chart_ball1.configure_mesh().draw().unwrap();
        chart_ball1
            .draw_series(LineSeries::new(
                ball1_x.iter().zip(ball1_y.iter()).map(|(&x, &y)| (x, y)),
                &RED,
            ))
            .unwrap()
            .label("Ball1")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        // Plot trajectory for Ball 2
        let plot_ball2_filename = format!(
            "log/analyze/plot_trajectory_ball2_{}.png",
            filename.replace("log/", "").replace(".txt", "")
        );
        let mut root_ball2 =
            BitMapBackend::new(&plot_ball2_filename, (640, 480)).into_drawing_area();
        root_ball2.fill(&WHITE).unwrap();
        let (min_x2, max_x2) = (
            *ball2_x
                .iter()
                .fold(&f64::INFINITY, |a, b| if b < a { b } else { a }),
            *ball2_x
                .iter()
                .fold(&f64::NEG_INFINITY, |a, b| if b > a { b } else { a }),
        );
        let (min_y2, max_y2) = (
            *ball2_y
                .iter()
                .fold(&f64::INFINITY, |a, b| if b < a { b } else { a }),
            *ball2_y
                .iter()
                .fold(&f64::NEG_INFINITY, |a, b| if b > a { b } else { a }),
        );
        let mut chart_ball2 = ChartBuilder::on(&root_ball2)
            .caption("Ball 2 Trajectory", ("sans-serif", 30).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(min_x2..max_x2, min_y2..max_y2)
            .unwrap();
        chart_ball2.configure_mesh().draw().unwrap();
        chart_ball2
            .draw_series(LineSeries::new(
                ball2_x.iter().zip(ball2_y.iter()).map(|(&x, &y)| (x, y)),
                &GREEN,
            ))
            .unwrap()
            .label("Ball2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));
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
