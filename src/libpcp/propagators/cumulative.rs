// Copyright 2016 Pierre Talbot (IRCAM)

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
use kernel::Trilean::*;
use propagators::*;
use propagation::*;
use propagation::events::*;
use term::ops::*;
use term::expr_inference::ExprInference;
use gcollections::ops::*;
use gcollections::IntervalKind;
use std::fmt::{Formatter, Debug, Error};
use num::traits::Num;
use num::PrimInt;
use std::ops::{Add, Sub};
use std::marker::PhantomData;

pub struct Cumulative<V, VStore>
{
  starts: Vec<Box<V>>,
  durations: Vec<Box<V>>,
  resources: Vec<Box<V>>,
  capacity: Box<V>,
  vstore_phantom: PhantomData<VStore>
}

impl<V, VStore> Cumulative<V, VStore>
{
  pub fn new(starts: Vec<Box<V>>, durations: Vec<Box<V>>,
   resources: Vec<Box<V>>, capacity: Box<V>) -> Self
  {
    assert_eq!(starts.len(), durations.len());
    assert_eq!(starts.len(), resources.len());
    Cumulative {
      starts: starts,
      durations: durations,
      resources: resources,
      capacity: capacity,
      vstore_phantom: PhantomData
    }
  }
}

impl<V, BV, VStore, Domain> Cumulative<V, VStore> where
  V: ExprInference<Output=Domain>,
  V: ViewDependencies<FDEvent>,
  V: StoreMonotonicUpdate<VStore, Domain>,
  V: StoreRead<VStore, Value=Domain>,
  V: Clone + 'static,
  Domain: Bounded<Bound=BV> + Add<BV, Output=Domain> + Clone + Sub<BV, Output=Domain>,
  Domain: Empty + ShrinkLeft<BV> + ShrinkRight<BV> + IntervalKind + 'static,
  BV: PrimInt + 'static
{
  // Decomposition described in `Why cumulative decomposition is not as bad as it sounds`, Schutt and al., 2009.
  // forall( j in tasks ) (
  //   c >= r[j] + sum( i in tasks where i != j ) (
  //     bool2int( s[i] <= s[j] /\ s[j] < s[i] + d[i] ) * r[i]));
  pub fn join<CStore>(&self, vstore: &mut VStore, cstore: &mut CStore) where
    CStore: Alloc<Box<PropagatorConcept<VStore, FDEvent> + 'static>>
  {
    let tasks = self.starts.len();
    for j in 0..tasks {
      for i in 0..tasks {
        if i != j {
          // bool2int(s[i] <= s[j] /\ s[j] < s[i] + d[i])
          cstore.alloc(box x_leq_y(*self.starts[i], *self.starts[j]));
        }
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use propagation::events::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn cumulative_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_11 = (10,11).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom1_10 = (1,10).to_interval();
    let dom5_10 = (5,10).to_interval();
    let dom6_10 = (6,10).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom2_2 = (2,2).to_interval();

    culmulative_test_one(1, dom0_10, dom0_10, dom0_10,
      Unknown, Unknown, vec![(0, Bound), (1, Bound), (2, Bound)], true);
    culmulative_test_one(2, dom10_11, dom5_15, dom5_15,
      Unknown, True, vec![(0, Assignment), (1, Assignment), (2, Assignment)], true);
    culmulative_test_one(3, dom10_20, dom1_1, dom1_1,
      True, True, vec![], true);
    culmulative_test_one(4, dom1_1, dom1_1, dom1_1,
      False, False, vec![], false);
    culmulative_test_one(5, dom2_2, dom1_1, dom1_1,
      False, False, vec![], false);
    culmulative_test_one(6, dom6_10, dom5_10, dom1_10,
      Unknown, Unknown, vec![(0, Bound), (1, Bound), (2, Bound)], true);
  }

  fn culmulative_test_one(test_num: u32,
    x: Interval<i32>, y: Interval<i32>, z: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    trinary_propagator_test(test_num, XGreaterYPlusZ::new, x, y, z, before, after, delta_expected, propagate_success);
  }
}
