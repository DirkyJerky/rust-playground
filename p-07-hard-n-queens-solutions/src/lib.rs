mod n_queens;

use n_queens::NQueensSolver;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test a queens solution for a given size and expected total count
    macro_rules! test_queens {
        ($n:expr, $total:literal) => {
            assert_eq!($total, NQueensSolver::<{ $n }>::all_solutions().count());
        };
    }

    /// Test a set of expect queens solution counts, by matching an array of expected counts to queen board sizes, starting at 0 and incrementing by one
    macro_rules! test_queens_range {
        // Entry point, calls recursive step
        ($($val:expr),* $(,)?) => {
            test_queens_range!(@step 0; $($val),*)
        };

        // Nothing left to process
        (@step $idx:expr;) => {};

        // Recursive step
        (@step $idx:expr; $head:expr $(, $tail:expr)*) => {
            // Test queens at idx and expected value
            test_queens!($idx, $head);

            // Recursive call
            test_queens_range!(@step $idx + 1; $($tail),*);
        };
    }

    #[test]
    fn test_counts() {
        // Source: https://oeis.org/A000170
        test_queens_range!(1, 1, 0, 0, 2, 10, 4, 40, 92, 352, 724, 2680, 14200);
    }
}
