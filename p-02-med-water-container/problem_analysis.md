# Analysis

The purpose of this file is to analyze the provided problem to support an optimal solution.

## Naive solution

1. Iterate over all combinations of indexes (i,j), 0 <= i < j < n
2. Compute the total volume of each combination (total = `(j-i) * input[i] * input[j]`)
3. Find the maximum volume, return

Runtime: O(n^2)

## Optimization #1 - Filtering out suboptimal left/right container sides, by comparison scans.

We can notice there will be many edges under consideration that can never be part of the optiomal solution.

By removing the suboptimal edges from the list of edges we consider for the leftmost/rightmost container walls, we are able to reduce the scan time to `1/4th` of the original naive scan time at _worst_, and often in much less time than that, given non-uniform input conditions.

We can perform this filtering by considering that when we evaluate any particular edge pairing for optimality, there may be edges outside of the current pairing that the container could "expand" to and would guarentee an increase to the container's size. In these situations, the current edge that can be "expanded" beyond will never be a part of optimal solution's left or right edge, respectively (depending on which direction it got expanded from).

We can build a this list beforehand, by scanning the full list from left-to-right and vice versa. Any edge that has a equal-or-larger edge to the left of it cannot be the optimal solution's left side wall, and likewise from the right.

From this filtering, we can accertain the following:

- Each edge will be at most part of one non-suboptional list, either the left or the right. With the exception of at most 1 edge, which can be a part of both lists. This is true, provable by contrapositive: If there did exist two seperate edges that are a part of both lists, then atleast one edge would be bigger or equal to the other edge, and would eliminate the other edge's eligibility in one of the lists, which breaks the prior assumption that both edges were on both lists.
- The worst case situation arises when both lists are roughly equal in size, and each edge is contained in at least one list. This is because if there existed a situation where the lists are unequal in size, the maximum amount of comparisons that can occur between the two lists would necessasrily be lesser (i.e. 8*2 < 7*3 < 6*4 < 5*5)
- It follows that the full set of combinations between these two lists is at most ~O(n/4) the original combination sets at worse.
