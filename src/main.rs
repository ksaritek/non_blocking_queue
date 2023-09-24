mod queue;

use std::{sync::Arc, thread};

fn main() {
    let queue = Arc::new(queue::Queue::new());

    // Spawn some threads for enqueueing
    let enqueuing_threads: Vec<_> = (0..5)
        .map(|i| {
            let q = queue.clone();
            thread::spawn(move || {
                q.enqueue(i);
            })
        })
        .collect();

    // Wait for enqueuing threads to finish
    for handle in enqueuing_threads {
        handle.join().unwrap();
    }

    // Spawn some threads for dequeuing
    let dequeuing_threads: Vec<_> = (0..5)
        .map(|_| {
            let q = queue.clone();
            thread::spawn(move || {
                let item = q.dequeue();
                match item {
                    Some(data) => println!("Dequeued: {}", data),
                    None => println!("Queue is empty."),
                }
            })
        })
        .collect();

    // Wait for dequeuing threads to finish
    for handle in dequeuing_threads {
        handle.join().unwrap();
    }
}
