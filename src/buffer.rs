pub struct Buffer<T: Copy> {
    inner: Vec<T>,
    pub size: usize,
    size_f32: f32,
    pub cursor: usize,
}

impl<T: Copy + std::fmt::Display> Buffer<T> {
    pub fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
            size,
            size_f32: size as f32,
            cursor: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.inner.len() < self.size {
            self.inner.push(value);
        } else {
            let _ = std::mem::replace(&mut self.inner[self.cursor], value);
        }
        self.cursor += 1;
    }

    pub fn as_full_slice(&mut self) -> &[T] {
        let _ = &mut self
            .inner
            .splice(
                self.cursor..,
                std::iter::repeat_n(
                    *self.inner.get(self.cursor - 1).unwrap(),
                    self.size - self.cursor,
                ),
            )
            .for_each(drop);

        &self.inner
    }

    pub fn percent_full(&self) -> f32 {
        (self.cursor as f32) / self.size_f32
    }

    pub fn has_capacity(&self) -> bool {
        self.cursor < self.size
    }
}
