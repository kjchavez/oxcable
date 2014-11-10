//! A generic ring buffer
//!
//! A ring buffer can continue adding data on to the end of itself indefinitely.
//! However, it has a limited capacity; when that capacity is reached, it will
//! remove the oldest data to make space.

#![unstable]

use std::clone::Clone;
use std::vec::Vec;

use core::Time;


/// A generic ring buffer
pub struct RingBuffer<T: Clone> {
    buf: Vec<T>,
    capacity: uint,
    size: uint,
    start_t: Time,
    end_t: Time,
}

impl<T: Clone> RingBuffer<T> {
    /// Returns an empty ring buffer that can hold at most `capacity` elements.
    pub fn new(capacity: uint) -> RingBuffer<T> {
        RingBuffer { 
            buf: Vec::with_capacity(capacity), 
            capacity: capacity,
            size: 0, 
            start_t: 0, 
            end_t: 0 
        }
    }

    /// Attempts to return the data stored at time `t`. If the requested time is
    /// not in the buffer, instead returns `None`.
    pub fn get(&self, t: Time) -> Option<T> {
        if self.start_t <= t && t < self.end_t {
            Some(self.buf[(t % self.capacity as Time) as uint].clone())
        } else {
            None
        }
    }

    /// Pushes the supplied data onto the end of the buffer. If the buffer is
    /// full, this will overwrite the oldest data.
    pub fn push(&mut self, data: T) {
        if self.size < self.capacity {
            self.buf.push(data);
            self.size += 1;
            self.end_t += 1;
        } else {
            self.buf[(self.end_t % self.capacity as Time) as uint] = data;
            self.start_t += 1;
            self.end_t += 1;
        }
    }

    /// Attempts to change the value at time `t` to `data`. If the requested
    /// time is not in the buffer, instead returns `Err`.
    pub fn update(&mut self, t: Time, data: T) -> Result<(),()> {
        if self.start_t <= t && t < self.end_t {
            self.buf[(t % self.capacity as Time) as uint] = data;
            Ok(())
        } else {
            Err(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn test_push() {
        let mut rb = RingBuffer::<int>::new(2);

        rb.push(13);
        assert_eq!(rb.size, 1);
        assert_eq!(rb.start_t, 0);
        assert_eq!(rb.end_t, 1);
        assert_eq!(rb.buf[0], 13);

        rb.push(7);
        assert_eq!(rb.size, 2);
        assert_eq!(rb.start_t, 0);
        assert_eq!(rb.end_t, 2);
        assert_eq!(rb.buf[0], 13);
        assert_eq!(rb.buf[1], 7);

        rb.push(3);
        assert_eq!(rb.size, 2);
        assert_eq!(rb.start_t, 1);
        assert_eq!(rb.end_t, 3);
        assert_eq!(rb.buf[0], 3);
        assert_eq!(rb.buf[1], 7);
    }

    #[test]
    fn test_get() {
        let mut rb: RingBuffer<int> = RingBuffer { 
            buf: vec![7,13], 
            capacity: 2, 
            size: 2,
            start_t: 7,
            end_t: 9
        };

        // Test with odd start
        assert_eq!(rb.get(6), None);
        assert_eq!(rb.get(7), Some(13));
        assert_eq!(rb.get(8), Some(7));
        assert_eq!(rb.get(9), None);

        // Test with even start
        rb.start_t = 6; rb.end_t = 8;
        assert_eq!(rb.get(5), None);
        assert_eq!(rb.get(6), Some(7));
        assert_eq!(rb.get(7), Some(13));
        assert_eq!(rb.get(8), None);
    }

    #[test]
    fn test_update() {
        let mut rb: RingBuffer<int> = RingBuffer { 
            buf: vec![7,13], 
            capacity: 2, 
            size: 2,
            start_t: 7,
            end_t: 9
        };

        // Test out of range
        assert_eq!(rb.update(6, 22), Err(()));
        assert_eq!(rb.update(9, 22), Err(()));

        // Test in range
        assert_eq!(rb.update(7, 22), Ok(()));
        assert_eq!(rb.buf[1], 22);
        assert_eq!(rb.update(8, 23), Ok(()));
        assert_eq!(rb.buf[0], 23);
    }
}
