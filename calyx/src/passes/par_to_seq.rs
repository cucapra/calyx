use crate::ir::traversal::{Action, Named, VisResult, Visitor};
use crate::ir::{self, LibrarySignatures};

#[derive(Default)]
/// Transforms all `par` into `seq`.
/// Useful for debugging problems with `par`.
///
/// # Example
/// 1. Collapses nested `seq`:
/// ```
/// par {
///     par { A; B }
///     C;
/// }
/// ```
/// into
/// ```
/// seq { seq { A; B } C; }
/// ```
///
/// To remove uneccessarily nested `par` blocks, run collapse control.
pub struct ParToSeq;

impl Named for ParToSeq {
    fn name() -> &'static str {
        "par-to-seq"
    }

    fn description() -> &'static str {
        "Transform `par` blocks to `seq`"
    }
}

impl Visitor for ParToSeq {
    /// Collapse par { par { A }; B } into par { A; B }.
    fn finish_par(
        &mut self,
        s: &mut ir::Par,
        _comp: &mut ir::Component,
        _c: &LibrarySignatures,
    ) -> VisResult {
        let par = ir::Control::seq(s.stmts.drain(..).collect());
        Ok(Action::Change(par))
    }
}
