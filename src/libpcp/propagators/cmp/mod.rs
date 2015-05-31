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

pub mod x_less_y;
pub mod x_eq_y;
pub mod x_neq_y;

use num::PrimInt;
use interval::ncollections::ops::*;
use variable::arithmetics::*;
pub use propagators::cmp::x_less_y::XLessY;
pub use propagators::cmp::x_eq_y::XEqY;
pub use propagators::cmp::x_neq_y::XNeqY;

pub fn x_greater_y<X, Y>(x: X, y: Y) -> XLessY<Y, X> {
  XLessY::new(y, x)
}

pub fn x_geq_y<X, Y, R, BX>(x: X, y: Y) -> XLessY<Y, Addition<X, BX>> where
  X: ExprInference<Output=R>,
  R: Bounded<Bound=BX>,
  BX: PrimInt
{
  x_greater_y(Addition::new(x, BX::one()), y)
}

pub fn x_leq_y<X, Y, R, BY>(x: X, y: Y) -> XLessY<X, Addition<Y, BY>> where
  Y: ExprInference<Output=R>,
  R: Bounded<Bound=BY>,
  BY: PrimInt
{
  XLessY::new(x, Addition::new(y, BY::one()))
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use solver::fd::event::*;
  use solver::fd::event::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_greater_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_11 = (10,11).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom5_11 = (5,11).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom9_9 = (9,9).to_interval();

    x_greater_y_test_one(1, dom0_10, dom0_10, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_greater_y_test_one(2, dom0_10, dom10_20, False, False, vec![], false);
    x_greater_y_test_one(3, dom5_15, dom10_20, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_greater_y_test_one(4, dom5_11, dom10_20, Unknown, True, vec![(0, Assignment), (1, Assignment)], true);
    x_greater_y_test_one(5, dom10_11, dom10_11, Unknown, True, vec![(0, Assignment), (1, Assignment)], true);
    x_greater_y_test_one(6, dom5_15, dom0_10, Unknown, Unknown, vec![], true);
    x_greater_y_test_one(7, dom11_20, dom0_10, True, True, vec![], true);
    x_greater_y_test_one(8, dom9_9, dom0_10, Unknown, True, vec![(1, Bound)], true);
  }

  fn x_greater_y_test_one(test_num: u32, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(test_num, x_greater_y, x, y, before, after, delta_expected, propagate_success);
  }

  #[test]
  fn x_geq_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom10_11 = (10,11).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom9_9 = (9,9).to_interval();

    x_geq_y_test_one(1, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_geq_y_test_one(2, dom0_10, dom10_20, Unknown, True, vec![(0, Assignment), (1, Assignment)], true);
    x_geq_y_test_one(3, dom5_15, dom10_20, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_geq_y_test_one(4, dom10_11, dom10_11, Unknown, Unknown, vec![], true);
    x_geq_y_test_one(5, dom5_15, dom0_10, Unknown, Unknown, vec![], true);
    x_geq_y_test_one(6, dom11_20, dom0_10, True, True, vec![], true);
    x_geq_y_test_one(7, dom9_9, dom0_10, Unknown, True, vec![(1, Bound)], true);
  }

  fn x_geq_y_test_one(test_num: u32, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(test_num, x_geq_y, x, y, before, after, delta_expected, propagate_success);
  }
}
