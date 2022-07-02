use crate::natives::CName;

const MAX_TRACE_SIZE: usize = 8;

#[derive(Debug, Default)]
pub struct StackFrameInfo {
    pub function: Option<CName>,
    pub class: Option<CName>,
}

#[derive(Debug)]
pub struct ReachedMaxTrace;

#[derive(Debug)]
pub struct StackTrace<const SIZE: usize = MAX_TRACE_SIZE> {
    frames: [StackFrameInfo; SIZE],
    size: usize,
}

impl<const SIZE: usize> StackTrace<SIZE> {
    pub fn try_push(&mut self, frame: StackFrameInfo) -> Result<(), ReachedMaxTrace> {
        if self.size < SIZE {
            self.frames[self.size] = frame;
            self.size += 1;
            Ok(())
        } else {
            Err(ReachedMaxTrace)
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &StackFrameInfo> {
        self.frames[..self.size].iter()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<const SIZE: usize> Default for StackTrace<SIZE> {
    fn default() -> Self {
        Self {
            frames: std::array::from_fn(|_| StackFrameInfo::default()),
            size: Default::default(),
        }
    }
}
