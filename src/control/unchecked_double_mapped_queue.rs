use std::ops::Range;

use vmcircbuffer::double_mapped_buffer::DoubleMappedBuffer;

pub struct UncheckedDoubleMappedQueue<T> {
    buffer: DoubleMappedBuffer<T>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T> UncheckedDoubleMappedQueue<T>
where
    T: Copy + Default,
{
    fn new(capacity: usize) -> Self {
        Self {
            buffer: DoubleMappedBuffer::new(capacity).expect("test"),
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn extend(&mut self, src: &[T]) {
        unsafe {
            let buffer_slice = self.buffer.slice_with_offset_mut(self.tail);
            buffer_slice[..src.len()].copy_from_slice(src);
        }

        self.tail += src.len();
        self.len += src.len();
        if self.tail > self.buffer.capacity() {
            self.tail = self.tail % self.buffer.capacity();
        }
    }

    fn drain(&mut self, count: usize) {
        self.len -= count;
        self.head += count;
        if self.head > self.buffer.capacity() {
            self.head = self.head % self.buffer.capacity();
        }
    }

    fn as_slice(&self) -> &[T] {
        unsafe {
            let buffer_slice = self.buffer.slice_with_offset(self.head);
            &buffer_slice[..self.len]
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use rand::random;

    use super::*;

    #[test]
    fn behaviour_mimicks_vecdeque_stress_test() {
        let test_cap = 10000;
        let test_cycles = 1000;

        let mut udmq: UncheckedDoubleMappedQueue<f32> = UncheckedDoubleMappedQueue::new(test_cap);
        let mut vdq: VecDeque<f32> = VecDeque::with_capacity(test_cap);

        let start_capacity = udmq.buffer.capacity();

        let mut cur_len = 0_usize;
        let mut cur_space = test_cap;

        for i in 0..test_cycles {
            let extend_num = (((rand::random::<f32>()) * cur_space as f32) as usize).max(cur_space);

            let random_vec: Vec<f32> = (0..extend_num).map(|f| rand::random::<f32>()).collect();

            udmq.extend(&random_vec);
            vdq.extend(&random_vec);

            cur_len += extend_num;
            cur_space -= extend_num;

            let drain_num = (((rand::random::<f32>()) * cur_len as f32) as usize).max(cur_len);

            udmq.drain(drain_num);
            vdq.drain(..drain_num);

            cur_len -= drain_num;
            cur_space += drain_num;

            {
                let udmq_whole_slice = udmq.as_slice();
                vdq.make_contiguous();
                let vdq_whole_slice = vdq.as_slices().0;
                assert_eq!(vdq_whole_slice, udmq_whole_slice);
            }
        }

        assert_eq!(start_capacity, udmq.buffer.capacity());
    }
}
