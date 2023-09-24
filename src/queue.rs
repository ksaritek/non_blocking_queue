use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Queue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

pub struct Node<T> {
    data: Option<Box<T>>,
    next: AtomicPtr<Node<T>>,
}

impl<T: std::default::Default> Queue<T> {
    pub fn new() -> Self {
        let node = Box::into_raw(Box::new(Node {
            data: Default::default(),
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));

        Queue {
            head: AtomicPtr::new(node),
            tail: AtomicPtr::new(node),
        }
    }

    pub fn enqueue(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data: Some(Box::new(data)),
            next: AtomicPtr::new(std::ptr::null_mut()),
        }));

        let mut tail = self.tail.load(Ordering::Relaxed);
        loop {
            let next = unsafe { (*tail).next.load(Ordering::Relaxed) };
            if next.is_null() {
                if self
                    .tail
                    .compare_exchange(tail, new_node, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    // Successfully added the new node, update the next pointer of the old tail
                    unsafe {
                        (*tail).next.store(new_node, Ordering::Relaxed);
                    }
                    return;
                }
            } else {
                // Another thread might have enqueued a new element, retry
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Relaxed);
            let tail = self.tail.load(Ordering::Relaxed);
            let next = unsafe { (*head).next.load(Ordering::Relaxed) };

            if head != self.head.load(Ordering::Relaxed) {
                continue; // Another thread modified the head, retry
            }

            if head == tail {
                if next.is_null() {
                    return None; // The queue is empty.
                }

                if self
                    .tail
                    .compare_exchange(tail, next, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    // Successfully updated the tail
                    // Continue to the next iteration to dequeue the element
                    continue;
                }
            }

            let data_option = unsafe { (*next).data.take() };

            if self
                .head
                .compare_exchange(head, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                // Successfully dequeued, take ownership of the data
                if let Some(data) = data_option {
                    return Some(*data);
                }
            }

            // If none of the compare_exchange operations succeeded, retry
        }
    }
}

mod tests {
    #[test]
    fn test_enqueue_dequeue() {
        let queue = super::Queue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
    }
}
