use std::{collections::VecDeque, sync::mpsc::Iter};

/// `N` queens placed on an NxN board, one queen by row
/// `board[n]` is the index of the column containing the queen in row `n`
#[derive(Clone, Copy, Debug)]
pub struct NQueensRepr<const N: usize> {
    pub board: [usize; N],
}

impl<const N: usize> Default for NQueensRepr<N> {
    fn default() -> Self {
        NQueensRepr { board: [0; N] }
    }
}

/// A backtracking solver for the N queens problem.
/// Progressively builds up the board, one row at a time
#[derive(Default, Clone, Copy, Debug)]
pub struct NQueensSolver<const N: usize> {
    /// The internal (possibly incomplete) board state at any given moment
    repr: NQueensRepr<N>,
    /// The number of rows that are considered "locked in" for this representation.
    /// All locked in rows are required to adhere to the the queen's capture rule:
    /// - No 2 queens may be in the same row (enforced by the data representation)
    /// - No 2 queens may be in the same column
    /// - No 2 queens may be in the same diagonal
    set_rows: usize,
}

#[derive(Debug)]
pub enum NQueenSolverEmission<const N: usize> {
    Entries(Vec<NQueensSolver<N>>),
    Solution(NQueensRepr<N>),
}

impl<const N: usize> NQueensSolver<N> {
    fn new() -> NQueensSolver<N> {
        Default::default()
    }

    fn append_row(&self, val: usize) -> NQueensSolver<N> {
        if self.set_rows >= N {
            panic!("Cannot append_row past capacity");
        }

        let mut new_solver = *self;

        new_solver.repr.board[new_solver.set_rows] = val;
        new_solver.set_rows += 1;

        new_solver
    }

    fn next_row_options(&self) -> impl Iterator<Item = usize> {
        (0..N).rev().into_iter().filter(|n| {
            for row_to_check_against in 0..self.set_rows {
                let row_diff = self.set_rows - row_to_check_against;

                let conflicting_left_diagonal_position = n.checked_sub(row_diff);
                let conflicting_column_position = n;
                let conflicting_right_diagonal_position = n + row_diff;

                if let Some(conflicting_left_diagonal_position) = conflicting_left_diagonal_position
                    && self.repr.board[row_to_check_against] == conflicting_left_diagonal_position
                {
                    return false;
                }

                if self.repr.board[row_to_check_against] == *conflicting_column_position {
                    return false;
                }

                if self.repr.board[row_to_check_against] == conflicting_right_diagonal_position {
                    return false;
                }
            }
            true
        })
    }

    fn check_emit_solution(&self) -> Option<NQueensRepr<N>> {
        if self.set_rows == N {
            return Some(self.repr);
        } else {
            None
        }
    }

    fn expand_states(self) -> NQueenSolverEmission<N> {
        if let Some(soln) = self.check_emit_solution() {
            return NQueenSolverEmission::Solution(soln);
        } else {
            NQueenSolverEmission::Entries(
                self.next_row_options()
                    .map(|v| self.append_row(v))
                    .collect::<Vec<NQueensSolver<N>>>(),
            )
        }
    }

    pub fn all_solutions() -> impl Iterator<Item = NQueensRepr<N>> {
        NQueensSolverIterator::new()
    }
}

struct NQueensSolverIterator<const N: usize> {
    inner: Vec<NQueensSolver<N>>,
}

impl<const N: usize> NQueensSolverIterator<N> {
    fn new() -> NQueensSolverIterator<N> {
        let mut inner = Vec::new();
        inner.push(NQueensSolver::new());
        NQueensSolverIterator { inner }
    }
}

impl<const N: usize> Iterator for NQueensSolverIterator<N> {
    type Item = NQueensRepr<N>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(solver) = self.inner.pop() {
            // let new_states = solver.expand_states();
            // eprintln!("{solver:?} -> {new_states:?}");
            match solver.expand_states() {
                NQueenSolverEmission::Entries(entries) => self.inner.extend(entries),
                NQueenSolverEmission::Solution(soln) => return Some(soln),
            }
        }

        None
    }
}
