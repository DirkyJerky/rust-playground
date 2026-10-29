use std::{cmp::max, collections::HashSet};

#[derive(Debug)]
pub struct LongestSubstringWithoutRepeatingCharacters {
    chars: Vec<char>,
    // Inclusive
    left_cursor: usize,
    // Exclusive
    right_cursor: usize,
    chars_in_substring: HashSet<char>,
}

impl LongestSubstringWithoutRepeatingCharacters {
    pub fn new(from: String) -> Self {
        Self {
            chars: from.chars().collect(),
            left_cursor: 0,
            right_cursor: 0,
            chars_in_substring: HashSet::new(),
        }
    }

    fn step_left(&mut self) -> bool {
        if let Some(chr) = self.chars.get(self.left_cursor) {
            self.chars_in_substring.remove(chr);
            self.left_cursor += 1;
            true
        } else {
            false
        }
    }

    fn grow_right(&mut self) {
        while let Some(chr) = self.chars.get(self.right_cursor) {
            if self.chars_in_substring.contains(chr) {
                break;
            } else {
                self.chars_in_substring.insert(*chr);
                self.right_cursor += 1;
            }
        }
    }

    fn substring_size(&mut self) -> usize {
        self.right_cursor.saturating_sub(self.left_cursor)
    }

    pub fn solve(mut self) -> usize {
        let mut max_found = 0;

        self.grow_right();
        max_found = max(max_found, self.substring_size());

        loop {
            let did_grow = self.step_left();
            if !did_grow {
                break;
            }

            self.grow_right();

            max_found = max(max_found, self.substring_size());
        }

        max_found
    }
}
