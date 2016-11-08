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

use term::ExprInference;
use num::{Zero, One};
use term::ops::*;
use propagation::ops::*;
use gcollections::ops::*;
use interval::ops::Range;
use std::cmp::PartialOrd;
use std::fmt::{Formatter, Debug, Error};
use std::marker::PhantomData;

/// x = bool2int(c)
#[derive(Clone, Copy)]
pub struct Bool2Int<DomX, P>
{
  x: PhantomData<DomX>,
  p: P,
}

impl<DomX, P> ExprInference for Bool2Int<DomX, P>
{
  type Output = DomX;
}

impl<DomX, P> Bool2Int<DomX, P> {
  pub fn new(p: P) -> Self {
    Bool2Int {
      x: PhantomData,
      p: p
    }
  }
}

impl<DomX, P> Debug for Bool2Int<DomX, P> where
  P: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!("bool2int({:?})", self.p))
  }
}

impl<DomX, BX, P, Store> StoreMonotonicUpdate<Store, DomX> for Bool2Int<DomX, P> where
  DomX: Bounded<Bound=BX> + IsSingleton,
  BX: Zero + One + PartialOrd + PartialEq,
  P: Propagator<Store>
{
  fn update(&mut self, store: &mut Store, value: DomX) -> bool {
    if value.is_singleton() {
      if value.lower() == BX::one() {
        self.p.propagate(store)
      }
      else {
        assert!(value.lower().is_zero(),
          "bool2int view can only be updated with a 'integer boolean' value: 0 or 1.");
        // We can not propagate the inverse of `P` (we do not know what it is).
        false
      }
    }
    else { true }
  }
}

impl<DomX, BX, P, Store> StoreRead<Store> for Bool2Int<DomX, P> where
  DomX: Bounded<Bound=BX> + Range<BX> + Singleton<BX>,
  BX: Zero + One + PartialOrd,
  P: Subsumption<Store>
{
  type Value = DomX;
  fn read(&self, store: &Store) -> Self::Value {
    use kernel::trilean::Trilean::*;
    match self.p.is_subsumed(store) {
      True => DomX::singleton(BX::one()),
      False => DomX::singleton(BX::zero()),
      Unknown => DomX::new(BX::zero(), BX::one())
    }
  }
}

impl<DomX, P, Event> ViewDependencies<Event> for Bool2Int<DomX, P> where
  P: PropagatorDependencies<Event>
{
  fn dependencies(&self, _event: Event) -> Vec<(usize, Event)> {
    self.p.dependencies()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use gcollections::ops::*;
  use propagators::XEqY;
  use kernel::Alloc;
  use variable::VStoreFD;
  use term::ops::*;
  use interval::interval::*;

  type VStore = VStoreFD;

  fn bool2int_read_x_eq_y(narrow_x: Interval<i32>, narrow_y: Interval<i32>, expected: Interval<i32>) {
    let dom0_10 = (0,10).to_interval();
    let dom0_1 = (0,1).to_interval();
    let mut store = VStore::empty();
    let mut x = store.alloc(dom0_10);
    let mut y = store.alloc(dom0_10);

    // z = bool2int(x == y)
    let z: Bool2Int<Interval<i32>, _> = Bool2Int::new(XEqY::new(x, y));
    assert_eq!(z.read(&store), dom0_1);

    x.update(&mut store, narrow_x);
    y.update(&mut store, narrow_y);
    assert_eq!(z.read(&store), expected);
  }

  #[test]
  fn bool2int_read() {
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();
    let dom0_0 = (0,0).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom0_1 = (0,1).to_interval();

    bool2int_read_x_eq_y(dom10_10, dom10_10, dom1_1);
    bool2int_read_x_eq_y(dom9_9, dom10_10, dom0_0);
    bool2int_read_x_eq_y(dom9_10, dom9_10, dom0_1);
  }

  fn bool2int_update_x_eq_y(dom_x: Interval<i32>, dom_y: Interval<i32>,
    narrow_z: Interval<i32>, expected_x: Interval<i32>, expected: bool)
  {
    let dom0_1 = (0,1).to_interval();
    let mut store = VStore::empty();
    let x = store.alloc(dom_x);
    let y = store.alloc(dom_y);

    // z = bool2int(x == y)
    let mut z: Bool2Int<Interval<i32>, _> = Bool2Int::new(XEqY::new(x, y));
    if expected {
      assert_eq!(z.read(&store), dom0_1);
    }

    let res_z = z.update(&mut store, narrow_z);
    assert_eq!(res_z, expected);
    assert_eq!(x.read(&store), expected_x);
  }

  #[test]
  fn bool2int_update() {
    let dom1_1 = (1,1).to_interval();
    let dom0_1 = (0,1).to_interval();
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();

    bool2int_update_x_eq_y(dom9_10, dom10_10, dom1_1, dom10_10, true);
    bool2int_update_x_eq_y(dom9_10, dom9_10, dom1_1, dom9_10, true);
    bool2int_update_x_eq_y(dom9_9, dom10_10, dom1_1, dom9_9, false);
    bool2int_update_x_eq_y(dom9_10, dom9_10, dom0_1, dom9_10, true);
  }

  #[test]
  #[should_panic]
  fn bool2int_panic_update() {
    let dom10_10 = (10,10).to_interval();
    let dom9_9 = (9,9).to_interval();
    let dom9_10 = (9,10).to_interval();
    let dom0_0 = (0,0).to_interval();
    bool2int_update_x_eq_y(dom9_10, dom10_10, dom0_0, dom9_9, true);
  }
}