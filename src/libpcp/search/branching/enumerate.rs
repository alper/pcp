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

use kernel::*;
use search::branching::*;
use search::branching::branch::*;
use search::space::*;
use variable::ops::*;
use term::*;
use propagators::cmp::*;
use propagation::concept::*;
use propagation::events::*;
use gcollections::ops::*;
use gcollections::*;
use std::ops::*;
use concept::*;

pub struct Enumerate;

// See discussion about type bounds: https://github.com/ptal/pcp/issues/11
impl<VStore, CStore, R, Domain, Bound> Distributor<Space<VStore, CStore, R>, Bound> for Enumerate where
  VStore: Freeze + Iterable<Item=Domain> + Index<usize, Output=Domain> + MonotonicUpdate,
  VStore: AssociativeCollection<Location=Identity<Domain>>,
  CStore: Freeze,
  CStore: Alloc + Collection<Item=Box<PropagatorConcept<VStore, FDEvent>>>,
  Domain: IntDomain<Item=Bound> + 'static,
  Bound: IntBound + 'static,
  R: FreezeSpace<VStore, CStore> + Snapshot<State=Space<VStore, CStore, R>>
{
  fn distribute(&mut self, space: Space<VStore, CStore, R>, var_idx: usize, val: Bound) ->
    (<Space<VStore, CStore, R> as Freeze>::FrozenState, Vec<Branch<Space<VStore, CStore, R>>>)
  {
    let x = Identity::<Domain>::new(var_idx);
    let v = Constant::new(val);

    let x_eq_v = XEqY::new(x.clone(), v.clone());
    let x_neq_v = XNeqY::new(x, v);

    Branch::distribute(space,
      vec![
        Box::new(move |space: &mut Space<VStore, CStore, R>| {
          space.cstore.alloc(box x_eq_v);
        }),
        Box::new(move |space: &mut Space<VStore, CStore, R>| {
          space.cstore.alloc(box x_neq_v);
        })
      ]
    )
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use search::branching::binary_split::test::test_distributor;
  use search::branching::MinVal;

  #[test]
  fn binary_split_distribution() {
    let vars = vec![(1,10),(2,4),(1,2)];
    test_distributor(Enumerate, MinVal, 0,
      vars.clone(),
      vec![(1,1),(2,10)]
    );
    test_distributor(Enumerate, MinVal, 1,
      vars.clone(),
      vec![(2,2),(3,4)]
    );
    test_distributor(Enumerate, MinVal, 2,
      vars.clone(),
      vec![(1,1),(2,2)]
    );
  }

  #[test]
  #[should_panic]
  fn binary_split_impossible_distribution() {
    test_distributor(Enumerate, MinVal, 0,
      vec![(1,1)],
      vec![]
    );
  }

  #[test]
  #[should_panic]
  fn binary_split_impossible_distribution_2() {
    test_distributor(Enumerate, MinVal, 2,
      vec![(1,5),(2,4),(4,4)],
      vec![]
    );
  }
}