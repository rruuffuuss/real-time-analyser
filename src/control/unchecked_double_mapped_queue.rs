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
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: DoubleMappedBuffer::new(capacity).expect("test"),
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn extend(&mut self, src: &[T]) {
        unsafe {
            let buffer_slice = self.buffer.slice_with_offset_mut(self.tail);
            buffer_slice[..src.len()].copy_from_slice(src);
        }

        self.tail += src.len();
        self.len += src.len();
        if self.tail >= self.buffer.capacity() {
            self.tail = self.tail % self.buffer.capacity();
        }
    }

    pub fn push_back(&mut self, src: T) {
        //self.extend(&[src]); works but this fn is so hot in fir_filter's half_band_into_udmq that this is worth is
        unsafe {
            *self
                .buffer
                .slice_with_offset_mut(self.tail)
                .get_unchecked_mut(0) = src;
        }

        self.tail += 1;
        self.len += 1;

        if self.tail == self.buffer.capacity() {
            self.tail = 0;
        }
    }

    pub fn drain(&mut self, count: usize) {
        self.len -= count;
        self.head += count;
        if self.head >= self.buffer.capacity() {
            self.head = self.head % self.buffer.capacity();
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let buffer_slice = self.buffer.slice_with_offset(self.head);
            &buffer_slice[..self.len]
        }
    }
}

impl<T> From<&Vec<T>> for UncheckedDoubleMappedQueue<T>
where
    T: Copy + Default,
{
    fn from(value: &Vec<T>) -> Self {
        let mut new = Self::new(value.len());
        new.extend(&value);
        new
    }
}

impl<T> Clone for UncheckedDoubleMappedQueue<T>
where
    T: Copy + Default,
{
    fn clone(&self) -> Self {
        let new_buffer: DoubleMappedBuffer<T> =
            DoubleMappedBuffer::<T>::new(self.buffer.capacity()).unwrap();

        let mut new = Self {
            buffer: new_buffer,
            head: self.head,
            tail: self.head,
            len: 0,
        };

        new.extend(self.as_slice());
        new
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

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

    #[test]
    fn test_from_vec() {
        let v = vec![0.0, 1.3, 2.9, 123414.034, 00000.1637];

        let udmq = UncheckedDoubleMappedQueue::from(&v);

        assert_eq!(&v, udmq.as_slice());
    }

    #[test]
    fn test_clone() {
        let v = vec![0.0, 1.3, 2.9, 123414.034, 00000.1637];

        let udmq1 = UncheckedDoubleMappedQueue::from(&v);
        let udmq2 = udmq1.clone();

        assert_eq!(udmq1.as_slice(), udmq2.as_slice());
    }

    #[test]
    fn test_push() {
        let test_capacity = 1024;

        let mut udmq: UncheckedDoubleMappedQueue<f32> =
            UncheckedDoubleMappedQueue::new(test_capacity);

        let random_vec: Vec<f32> = (0..test_capacity).map(|f| rand::random::<f32>()).collect();

        random_vec.iter().for_each(|f| udmq.push_back(*f));

        assert_eq!(&random_vec, udmq.as_slice());

        udmq.drain(test_capacity / 2);

        let random_vec2: Vec<f32> = (0..test_capacity / 2)
            .map(|f| rand::random::<f32>())
            .collect();

        random_vec2.iter().for_each(|f| udmq.push_back(*f));

        let random_vec_combined: Vec<f32> = random_vec[test_capacity / 2..]
            .iter()
            .chain(random_vec2.iter())
            .map(|f| *f)
            .collect();

        assert_eq!(&random_vec_combined, udmq.as_slice());
    }
}
