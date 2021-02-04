use crate::analysis::GraphAnalysis;
use crate::errors::Error;
use crate::ir::traversal::{Action, Named, VisResult, Visitor};
use crate::ir::{self, LibrarySignatures};
use std::collections::HashSet;

/// Pass to check for common errors such as missing assignments to `done` holes
/// of groups.
pub struct SynthesisPapercut {
    /// Names of memory primitives
    memories: HashSet<ir::Id>,
}

impl Default for SynthesisPapercut {
    fn default() -> Self {
        let memories = ["std_mem_d1", "std_mem_d2", "std_mem_d3", "std_mem_d4"]
            .iter()
            .map(|&mem| mem.into())
            .collect();
        SynthesisPapercut { memories }
    }
}

impl Named for SynthesisPapercut {
    fn name() -> &'static str {
        "synthesis-papercut"
    }

    fn description() -> &'static str {
        "Detect common problems when targeting synthesis backends"
    }
}

impl Visitor for SynthesisPapercut {
    fn start(
        &mut self,
        comp: &mut ir::Component,
        _ctx: &LibrarySignatures,
    ) -> VisResult {
        let memory_cells = comp
            .cells
            .iter()
            .filter_map(|cell| {
                let cell = &cell.borrow();
                if let Some(parent) = cell.type_name() {
                    if self.memories.contains(&parent) {
                        let has_external = cell.get_attribute("external");
                        if has_external.is_none() {
                            return Some(cell.name.clone());
                        }
                    }
                }
                None
            })
            .collect::<HashSet<_>>();

        if memory_cells.is_empty() {
            return Ok(Action::Stop);
        }

        let has_mem_parent =
            |p: &ir::Port| memory_cells.contains(&p.get_parent_name());
        let analysis =
            GraphAnalysis::from(&*comp).edge_induced_subgraph(|p1, p2| {
                has_mem_parent(p1) || has_mem_parent(p2)
            });

        for mem in memory_cells {
            println!("{}", mem);
            let cell = comp.find_cell(&mem).unwrap();
            let read_port = cell.borrow().get("read_data");
            if analysis.reads_from(&*read_port.borrow()).next().is_none() {
                return Err(Error::Papercut(
                    format!(
                        "Only reads performed on memory `{}'. Synthesis tools with remove this memory. Add @external(1) to cell to turn this into an interface memory.",
                        mem.to_string()
                    ),
                    mem,
                ));
            }
            let write_port = cell.borrow().get("write_data");
            if analysis.writes_to(&*write_port.borrow()).next().is_none() {
                return Err(Error::Papercut(
                    format!(
                        "Only writes performed on memory `{}'. Synthesis tools with remove this memory. Add @external(1) to cell to turn this into an interface memory.",
                        mem.to_string()
                    ),
                    mem,
                ));
            }
        }
        Ok(Action::Stop)
    }
}
