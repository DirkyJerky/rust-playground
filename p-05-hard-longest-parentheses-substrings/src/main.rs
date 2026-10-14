use std::collections::{HashMap, VecDeque};

/// A left or right parenthesis
#[derive(PartialEq, Eq)]
enum Parenthesis {
    LEFT,
    RIGHT,
}

/// Encodes all the state necessary to solve the problem efficiently via dynamic programming
struct ParenthesisSubstringSolver {
    /// The size of the original string
    size: usize,
    /// The parsed string
    arr: Vec<Parenthesis>,
    /// A map containing exhaustive coverage of all tracked valid substrings, mapping `[left_idx]` -> `right_idx`.
    /// `left_idx` is inclusive.
    /// `right_idx` is exclusive.
    left_idx_map: HashMap<usize, usize>,
    /// A map containing exhaustive coverage of all tracked valid substrings, mapping `[right_idx]` -> `left_idx`.
    /// It is assumed that all entries in `left_idx_map` have one and exactly one entry in `right_idx_map`.
    right_idx_map: HashMap<usize, usize>,
    /// A queue of substrings to check for possible growth, indexed by (left_idx, right_idx) pairs
    /// A substring can "grow" one of two ways:
    /// - Merging with a valid substring that is adjacent on the left or right side
    /// - Growing into a surrounding parenthesis pair that exactly encapsulates the current substring
    resolve_queue: VecDeque<(usize, usize)>,
}

impl ParenthesisSubstringSolver {
    pub fn new(parenthesis_str: &str) -> ParenthesisSubstringSolver {
        let arr: Vec<Parenthesis> = parenthesis_str
            .chars()
            .map(|char| match char {
                '(' => Parenthesis::LEFT,
                ')' => Parenthesis::RIGHT,
                _ => panic!("Unexpected character {char}"),
            })
            .collect();

        let size: usize = arr.len();

        let mut left_idx_map: HashMap<usize, usize> = HashMap::with_capacity(size);
        let mut right_idx_map: HashMap<usize, usize> = HashMap::with_capacity(size);
        let mut resolve_queue: VecDeque<(usize, usize)> = VecDeque::with_capacity(size);

        for i in 0..=size {
            left_idx_map.insert(i, i);
            right_idx_map.insert(i, i);
            resolve_queue.push_back((i, i));
        }

        ParenthesisSubstringSolver {
            size,
            arr,
            left_idx_map,
            right_idx_map,
            resolve_queue,
        }
    }

    fn clear_entry(&mut self, left_idx: usize, right_idx: usize) {
        self.left_idx_map.remove(&left_idx);
        self.right_idx_map.remove(&right_idx);
    }

    fn add_entry(&mut self, left_idx: usize, right_idx: usize) {
        self.left_idx_map.insert(left_idx, right_idx);
        self.right_idx_map.insert(right_idx, left_idx);
    }

    fn check_existance(&mut self, left_idx: usize, right_idx: usize) -> bool {
        self.left_idx_map.get(&left_idx) == Some(&right_idx)
    }

    /// Check if this substring can be merged with an existing substring adjacent to the left
    /// Returns `Some((left_idx, right_idx))` if the substring was expanded
    fn check_merge_left(
        &mut self,
        (left_idx, right_idx): (usize, usize),
    ) -> Option<(usize, usize)> {
        // If this substring is not in the idx map, then it was removed by a prior step, and can be ignored
        if !self.check_existance(left_idx, right_idx) {
            return None;
        }
        // Prevent self-referential loops
        if left_idx == right_idx {
            return None;
        }
        // Get the new left idx to expand to, if it exists
        let new_left_idx = *self.right_idx_map.get(&left_idx)?;

        self.clear_entry(left_idx, right_idx);
        self.clear_entry(new_left_idx, left_idx);
        self.add_entry(new_left_idx, right_idx);

        Some((new_left_idx, right_idx))
    }

    /// Check if this substring can be merged with an existing substring adjacent to the right
    /// Returns `Some((left_idx, right_idx))` if the substring was expanded
    fn check_merge_right(
        &mut self,
        (left_idx, right_idx): (usize, usize),
    ) -> Option<(usize, usize)> {
        // If this substring is not in the idx map, then it was removed by a prior step, and can be ignored
        if !self.check_existance(left_idx, right_idx) {
            return None;
        }
        // Prevent self-referential loops
        if left_idx == right_idx {
            return None;
        }
        // Get the new right idx to expand to, if it exists
        let new_right_idx = *self.left_idx_map.get(&right_idx)?;

        self.clear_entry(left_idx, right_idx);
        self.clear_entry(right_idx, new_right_idx);
        self.add_entry(left_idx, new_right_idx);

        Some((left_idx, new_right_idx))
    }

    /// Check if this substring can grow into one set of surrounding parenthesis
    fn check_growth(&mut self, (left_idx, right_idx): (usize, usize)) -> Option<(usize, usize)> {
        if !self.check_existance(left_idx, right_idx) {
            return None;
        }

        if left_idx > 0
            && right_idx < self.size
            && self.arr[left_idx - 1] == Parenthesis::LEFT
            && self.arr[right_idx] == Parenthesis::RIGHT
        {
            let new_left_idx = left_idx - 1;
            let new_right_idx = right_idx + 1;
            self.clear_entry(left_idx, right_idx);
            self.add_entry(new_left_idx, new_right_idx);
            Some((new_left_idx, new_right_idx))
        } else {
            None
        }
    }

    pub fn solve(mut self) -> usize {
        while let Some(pair) = self.resolve_queue.pop_front() {
            if let Some(new_pair) = self.check_merge_left(pair) {
                self.resolve_queue.push_back(new_pair);
            } else if let Some(new_pair) = self.check_merge_right(pair) {
                self.resolve_queue.push_back(new_pair);
            } else if let Some(new_pair) = self.check_growth(pair) {
                self.resolve_queue.push_back(new_pair);
            }
        }

        self.left_idx_map
            .into_iter()
            .map(|(left, right)| right - left)
            .max()
            .expect("left_idx_map should have values")
    }
}

fn longest_parenthesis_substring(str: &str) -> usize {
    ParenthesisSubstringSolver::new(str).solve()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cases() {
        let cases: [(&str, usize); _] = [
            ("(()", 2),
            ("())", 2),
            (")()())", 4),
            ("", 0),
            ("((()))))()(()())((()))(", 14),
        ];

        for (input, output) in cases {
            assert_eq!(
                longest_parenthesis_substring(input),
                output,
                "{input} -> {output}"
            )
        }
    }
}
