// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(plugin)]
#![plugin(pcp_lang)]

extern crate gcollections;
extern crate interval;
extern crate pcp;

#[cfg(test)]
mod test {
    use pcp::kernel::*;
    use pcp::propagation::events::*;
    use pcp::propagation::reactors::*;
    use pcp::propagation::schedulers::*;
    use pcp::propagation::store::Store as ConStore;
    use pcp::propagators::cmp::*;
    use pcp::propagators::distinct::*;
    use pcp::term::*;
    use pcp::variable::memory::*;
    use pcp::variable::store::Store as VarStore;

    use pcp::search::branching::binary_split::*;
    use pcp::search::branching::brancher::*;
    use pcp::search::branching::first_smallest_var::*;
    use pcp::search::engine::one_solution::*;
    use pcp::search::propagation::*;
    use pcp::search::search_tree_visitor::*;
    use pcp::search::space::*;

    use gcollections::ops::*;
    use gcollections::wrappers::*;
    use interval::interval::*;
    use interval::ops::*;

    type Domain = Interval<isize>;
    type VStore = VarStore<CopyMemory<Domain>, Domain, FDEvent>;
    type CStore = ConStore<VStore, FDEvent, IndexedDeps, RelaxedFifo>;
    type FDSpace = Space<VStore, CStore>;

    #[test]
    fn test_nqueens() {
        pcp! {
          let mut variables: VStore = VStore::empty();
          let mut constraints: CStore = CStore::empty();
          let n = 10usize;
          let mut queens = vec![];
          for _ in 0..n {
            let n: isize = n as isize;
            queens.push(#(variables <- 0..n));
          }
          for i in 0..n-1 {
            for j in i + 1..n {
              let a = i as isize;
              let b = j as isize;
              let ma = -a;
              let mb = -b;
              #{
                constraints <- queens[i] + a != queens[j] + b;
                constraints <- queens[i] + ma != queens[j] + mb;
              }
            }
          }
          #(constraints <- Distinct(queens));

          let space = FDSpace::new(variables, constraints);
          let mut search: OneSolution<_, Vector<_>, FDSpace> =
            OneSolution::new(Propagation::new(Brancher::new(FirstSmallestVar, BinarySplit)));
          search.start(&space);
          let (_, status) = search.enter(space);
          assert_eq!(status, Status::Satisfiable);
        }
        assert!(true);
    }
}
