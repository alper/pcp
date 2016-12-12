// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// The `AllSolution` combinator continuously calls its child until it returns `EndOfSearch`. You should use it with the `OneSolution` combinator.

use kernel::*;
use search::search_tree_visitor::*;
use search::search_tree_visitor::Status::*;

pub struct AllSolution<C> {
  child: C
}

impl<C> AllSolution<C>
{
  pub fn new(child: C) -> Self
  {
    AllSolution {
      child: child
    }
  }
}

impl<C, Space> SearchTreeVisitor<Space> for AllSolution<C> where
 Space: Freeze,
 C: SearchTreeVisitor<Space>
{
  fn start(&mut self, root: &Space) {
    self.child.start(root);
  }

  fn enter(&mut self, root: Space) -> (Space::FrozenState, Status<Space>) {
    let (mut immutable_state, mut status) = self.child.enter(root);
    while status != EndOfSearch {
      let state = immutable_state.unfreeze();
      let frozen_state = self.child.enter(state);
      immutable_state = frozen_state.0;
      status = frozen_state.1;
    }
    (immutable_state, status)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use search::test::*;
  use search::engine::one_solution::*;
  use search::propagation::*;
  use search::branching::binary_split::*;
  use search::branching::brancher::*;
  use search::branching::first_smallest_var::*;
  use gcollections::VectorStack;
  use gcollections::ops::*;

  #[test]
  fn example_nqueens() {
    for i in 1..10 {
      test_nqueens(i, EndOfSearch);
    }
  }

  fn test_nqueens(n: usize, expect: Status<FDSpace>) {
    let mut space = FDSpace::empty();
    nqueens(n, &mut space);

    let mut search: AllSolution<OneSolution<_, VectorStack<_>, FDSpace>> =
      AllSolution::new(OneSolution::new(Propagation::new(Brancher::new(FirstSmallestVar, BinarySplit))));
    search.start(&space);
    let (_, status) = search.enter(space);
    assert_eq!(status, expect);
  }
}