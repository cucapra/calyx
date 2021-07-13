use crate::ir::Guard;
use crate::ir::{
    self,
    traversal::{Action, Named, VisResult, Visitor},
    LibrarySignatures,
};

/// Canonicalizes the guard expression.
#[derive(Default)]
pub struct GuardCanonical;

impl Named for GuardCanonical {
    fn name() -> &'static str {
        "guard-canonical"
    }

    fn description() -> &'static str {
        "canonicalizes the guard expression"
    }
}

fn update_assigns(assigns: &mut [ir::Assignment]) {
    for assign in assigns {
        if let Guard::Port(p) = &(*assign.guard) {
            // 1'd1 ? r1.done
            if p.borrow().is_constant(1, 1) {
                assign.guard = Guard::True.into()
            }
            // r1.done ? 1'd1
            else if assign.src.borrow().is_constant(1, 1) {
                assign.src = p.clone(); //rc clone
                assign.guard = Guard::True.into();
            }
        }
    }
}

impl Visitor for GuardCanonical {
    fn start(
        &mut self,
        comp: &mut ir::Component,
        _ctx: &LibrarySignatures,
    ) -> VisResult {
        // For each group and continuous assignments, canonicalize guard
        // statements that has constant 1 as either a source or a guard.
        // # Example
        // ```
        // a[done] = 1'd1 ? r1.done
        //   -> a[done] = r1.done
        // a[done] = r1.done ? 1'd1
        //   -> a[done] = r1.done
        // ```
        for group in comp.groups.iter() {
            update_assigns(&mut group.borrow_mut().assignments[..]);
        }
        update_assigns(&mut comp.continuous_assignments[..]);

        Ok(Action::Stop)
    }
}
