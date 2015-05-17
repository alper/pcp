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

use kernel::DeepClone;
use variable::ops::*;
use variable::store::*;
use variable::arithmetics::identity::*;
use solver::event::*;
use solver::merge::*;
use solver::iterator::*;
use std::slice;
use std::collections::vec_map::{Drain, VecMap};
use interval::ncollections::ops::*;

pub struct DeltaStore<Event, Domain> {
  store: Store<Domain>,
  delta: VecMap<Event>
}

impl<Event, Domain> DeltaStore<Event, Domain>
{
  pub fn new() -> DeltaStore<Event, Domain> {
    DeltaStore {
      store: Store::new(),
      delta: VecMap::new()
    }
  }

  pub fn drain_delta<'a>(&'a mut self) -> Drain<'a, Event> {
    self.delta.drain()
  }
}

impl<Event, Domain> DeepClone for DeltaStore<Event, Domain> where
  Domain: Clone
{
  fn deep_clone(&self) -> Self {
    DeltaStore {
      store: self.store.deep_clone(),
      delta: VecMap::new()
    }
  }
}

impl<Event, Domain> VariableIterator for DeltaStore<Event, Domain> {
  type Variable = Domain;

  fn vars_iter<'a>(&'a self) -> slice::Iter<'a, Domain> {
    self.store.vars_iter()
  }
}

impl<Event, Domain> Cardinality for DeltaStore<Event, Domain>
{
  type Size = usize;

  fn size(&self) -> usize {
    self.store.size()
  }
}

impl<Event, Domain> Assign<Domain> for DeltaStore<Event, Domain> where
  Domain: Cardinality
{
  type Variable = Identity<Domain>;

  fn assign(&mut self, dom: Domain) -> Identity<Domain> {
    let var = self.store.assign(dom);
    self.delta.reserve_len(var.index());
    var
  }
}

impl<Event, Domain> MonotonicUpdate<usize, Domain> for DeltaStore<Event, Domain> where
  Domain: VarDomain+Clone,
  Event: MonotonicEvent<Domain> + Merge + Clone
{
  fn update(&mut self, key: usize, dom: Domain) -> bool {
    assert!(dom.is_subset(&self.store.read(key)), "Domain update must be monotonic.");
    if dom.is_empty() { false }
    else {
      if let Some(event) = Event::new(&dom, &self.store.read(key)) {
        let mut updated = false;
        if let Some(old) = self.delta.get_mut(&key) {
          *old = Merge::merge(old.clone(), event.clone());
          updated = true;
        }
        if !updated {
          self.delta.insert(key, event);
        }
        self.store.update(key, dom);
      }
      true
    }
  }
}

impl<Event, Domain> Read<usize> for DeltaStore<Event, Domain> where
  Domain: Clone
{
  type Value = Domain;
  fn read(&self, key: usize) -> Domain {
    self.store.read(key)
  }
}

#[cfg(test)]
pub mod test {
  use super::*;
  use variable::ops::*;
  use variable::arithmetics::identity::*;
  use solver::fd::event::*;
  use solver::fd::event::FDEvent::*;
  use interval::interval::*;
  use interval::ncollections::ops::*;

  type FDStore = DeltaStore<FDEvent, Interval<i32>>;

  fn test_op<Op>(source: Interval<i32>, target: Interval<i32>, delta_expected: Vec<FDEvent>, update_success: bool, op: Op) where
    Op: FnOnce(&FDStore, Identity<Interval<i32>>) -> Interval<i32>
  {
    let mut store = DeltaStore::new();
    let var = store.assign(source);

    let new = op(&store, var);
    assert_eq!(var.update(&mut store, new), update_success);
    assert_eq!(new, target);

    if update_success {
      let delta_expected = delta_expected.into_iter().map(|d| (var.index(), d)).collect();
      consume_delta(&mut store, delta_expected);
      assert_eq!(var.read(&store), target);
    }
  }

  fn test_binary_op<Op>(source1: Interval<i32>, source2: Interval<i32>, target: Interval<i32>, delta_expected: Vec<(usize, FDEvent)>, update_success: bool, op: Op) where
    Op: FnOnce(&FDStore, Identity<Interval<i32>>, Identity<Interval<i32>>) -> Interval<i32>
  {
    let mut store = DeltaStore::new();
    let var1 = store.assign(source1);
    let var2 = store.assign(source2);

    let new = op(&store, var1, var2);
    assert_eq!(var1.update(&mut store, new), update_success);
    assert_eq!(var2.update(&mut store, new), update_success);
    assert_eq!(new, target);

    if update_success {
      consume_delta(&mut store, delta_expected);
      assert_eq!(var1.read(&store), target);
      assert_eq!(var2.read(&store), target);
    }
  }

  pub fn consume_delta(store: &mut DeltaStore<FDEvent, Interval<i32>>, delta_expected: Vec<(usize, FDEvent)>) {
    let res: Vec<(usize, FDEvent)> = store.drain_delta().collect();
    assert_eq!(res, delta_expected);
    assert!(store.drain_delta().next().is_none());
  }

  #[test]
  fn var_update_test() {
    let dom0_10 = (0,10).to_interval();
    let dom0_9 = (0,5).to_interval();
    let dom1_10 = (5,10).to_interval();
    let dom1_9 = (1,9).to_interval();
    let dom0_0 = (0,0).to_interval();
    let empty = Interval::empty();

    var_update_test_one(dom0_10, dom0_10, vec![], true);
    var_update_test_one(dom0_10, empty, vec![], false);
    var_update_test_one(dom0_10, dom0_0, vec![Assignment], true);
    var_update_test_one(dom0_10, dom1_10, vec![Bound], true);
    var_update_test_one(dom0_10, dom0_9, vec![Bound], true);
    var_update_test_one(dom0_10, dom1_9, vec![Bound], true);
  }

  fn var_update_test_one(source: Interval<i32>, target: Interval<i32>, delta_expected: Vec<FDEvent>, update_success: bool) {
    test_op(source, target, delta_expected, update_success, |_,_| target);
  }

  #[test]
  fn var_shrink_bound() {
    let dom0_10 = (0,10).to_interval();

    var_shrink_lb_test_one(dom0_10, 0, vec![], true);
    var_shrink_lb_test_one(dom0_10, 10, vec![Assignment], true);
    var_shrink_lb_test_one(dom0_10, 1, vec![Bound], true);
    var_shrink_lb_test_one(dom0_10, 11, vec![], false);

    var_shrink_ub_test_one(dom0_10, 10, vec![], true);
    var_shrink_ub_test_one(dom0_10, 0, vec![Assignment], true);
    var_shrink_ub_test_one(dom0_10, 1, vec![Bound], true);
    var_shrink_ub_test_one(dom0_10, -1, vec![], false);
  }

  fn var_shrink_lb_test_one(source: Interval<i32>, target_lb: i32, delta_expected: Vec<FDEvent>, update_success: bool) {
    let expected_dom = (target_lb, source.upper()).to_interval();

    test_op(source, expected_dom, delta_expected, update_success,
      |store, var| var.read(store).shrink_left(target_lb));
  }

  fn var_shrink_ub_test_one(source: Interval<i32>, target_ub: i32, delta_expected: Vec<FDEvent>, update_success: bool) {
    let expected_dom = (source.lower(), target_ub).to_interval();

    test_op(source, expected_dom, delta_expected, update_success,
      |store, var| var.read(store).shrink_right(target_ub));
  }

  #[test]
  fn var_intersection_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_10 = (10,10).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom1_9 = (1,9).to_interval();

    var_intersection_test_one(dom0_10, dom10_20, dom10_10, vec![(0, Assignment), (1, Assignment)], true);
    var_intersection_test_one(dom0_10, dom1_9, dom1_9, vec![(0, Bound)], true);
    var_intersection_test_one(dom1_9, dom0_10, dom1_9, vec![(1, Bound)], true);
    var_intersection_test_one(dom0_10, dom11_20, Interval::empty(), vec![], false);
  }

  fn var_intersection_test_one(source1: Interval<i32>, source2: Interval<i32>, target: Interval<i32>, delta_expected: Vec<(usize, FDEvent)>, update_success: bool) {
    test_binary_op(source1, source2, target, delta_expected, update_success,
      |store, v1, v2| v1.read(store).intersection(&v2.read(store)));
  }
}