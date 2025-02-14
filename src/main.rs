use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod least_squares;
mod logger;

use least_squares::solve_least_squares;
use logger::Logger;

fn main() {
    // Create a logger
    let logger = Logger::new();
    let logger = Arc::new(logger);

    // Create a channel
    let (tx, rx) = mpsc::channel();

    // Spawn the producer thread
    let producer = thread::spawn(move || {
        loop {
            let start = Instant::now();

            // Solve a random least squares problem
            let result = solve_least_squares();

            let duration = start.elapsed();
            let time_taken_us = duration.as_micros();

            // Send the result and time taken to the consumer
            tx.send((result, time_taken_us)).unwrap();

            // Sleep to maintain 50Hz frequency
            thread::sleep(Duration::from_millis(20));
        }
    });

    // Spawn the consumer thread
    let consumer_logger = Arc::clone(&logger);
    let consumer = thread::spawn(move || {
        loop {
            // Receive the latest result and time taken
            if let Ok((result, time_taken_us)) = rx.try_recv() {
                println!("Received result: {:?}", result);
                println!("Time taken: {} us", time_taken_us);

                // Log the result and time taken
                consumer_logger.log(result.len(), &result, time_taken_us);
            }

            // Sleep to maintain 100Hz frequency
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Wait for both threads to finish
    producer.join().unwrap();
    consumer.join().unwrap();
}
