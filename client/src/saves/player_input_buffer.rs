use packets::NetplayBufferItem;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInputBuffer {
    buffer: VecDeque<(NetplayBufferItem, usize)>,
    len: usize,
}

impl Default for PlayerInputBuffer {
    fn default() -> Self {
        Self::new_with_delay(5)
    }
}

impl PlayerInputBuffer {
    pub fn new_with_delay(delay: usize) -> Self {
        let mut buffer = VecDeque::default();
        buffer.push_back((NetplayBufferItem::default(), delay));

        Self { buffer, len: delay }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_last(&mut self, input: NetplayBufferItem) {
        self.len += 1;

        if let Some((item, count)) = self.buffer.back_mut() {
            if *item == input {
                *count += 1;
                return;
            }
        }

        self.buffer.push_back((input, 1));
    }

    pub fn delete_last(&mut self) {
        let Some((_, count)) = self.buffer.back_mut() else {
            return;
        };

        self.len -= 1;
        *count -= 1;

        if *count == 0 {
            self.buffer.pop_back();
        }
    }

    pub fn peek_next(&self) -> Option<&NetplayBufferItem> {
        self.buffer.front().map(|(item, _)| item)
    }

    pub fn pop_next(&mut self) -> Option<NetplayBufferItem> {
        let (item, count) = self.buffer.front_mut()?;

        self.len -= 1;
        *count -= 1;

        if *count == 0 {
            self.buffer.pop_front().map(|(item, _)| item)
        } else {
            Some(item.clone())
        }
    }

    pub fn get(&self, mut index: usize) -> Option<&NetplayBufferItem> {
        self.buffer
            .iter()
            .find(move |(_, count)| {
                if *count > index {
                    return true;
                }

                index -= *count;
                false
            })
            .map(|(item, _)| item)
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.len = 0;
    }
}
