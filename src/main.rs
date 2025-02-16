use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod least_squares;
mod logger;
mod simulator;

use least_squares::solve_least_squares;
use logger::Logger;
use simulator::Simulator;

fn main() {
    // Create a logger
    let logger = Logger::new();
    let logger = Arc::new(logger);

    // Create a channel
    let (tx, rx) = mpsc::channel();

    // Configuration for the balls
    let ball1_config = (0.5, 1.0, 0.7, [0.0, 3.0, 0.0], [-0.1, 0.1, 0.2]);
    let ball2_config = (0.5, 1.0, 0.7, [0.0, 3.0, 1.0], [0.1, 0.3, -0.1]);

    // Spawn the producer thread
    let producer = thread::spawn(move || {
        let mut simulator = Simulator::new(ball1_config, ball2_config);

        loop {
            let (positions, time_taken_us) = simulator.step();

            // Send the result and time taken to the consumer
            tx.send((positions, time_taken_us)).unwrap();

            // Sleep to maintain 1000Hz frequency
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Spawn the consumer thread
    let consumer_logger = Arc::clone(&logger);
    let consumer = thread::spawn(move || {
        use nalgebra::DVector;
        loop {
            // Receive the latest result and time taken
            if let Ok((result, time_taken_us)) = rx.try_recv() {
                println!("Received result: {:?}", result);
                println!("Time taken: {} us", time_taken_us);

                // Convert Vec<Vector2<f32>> into a DVector<f64>
                let data: Vec<f64> = result
                    .iter()
                    .flat_map(|v| vec![v.x as f64, v.y as f64, v.z as f64])
                    .collect();
                let dvec = DVector::from_vec(data);
                consumer_logger.log(dvec.len(), &dvec, time_taken_us);
            }

            // Sleep to maintain 100Hz frequency
            thread::sleep(Duration::from_millis(10));
        }
    });

    // Wait for both threads to finish
    producer.join().unwrap();
    consumer.join().unwrap();
}
