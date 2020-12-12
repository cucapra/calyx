//! Statically infers the number of cycles for groups where the `done`
//! signal relies only on other `done` signals, and then inserts "static"
//! annotations with those inferred values. If there is an existing
//! annotation in a group that differs from an inferred value, this
//! pass will throw an error. If a group's `done` signal relies on signals
//! that are not only `done` signals, this pass will ignore that group.
use std::collections::HashMap;

use crate::analysis::{ReadWriteSet, GraphAnalysis};
use crate::errors::Error;
use crate::frontend::library::ast as lib;
use crate::ir;
use crate::ir::traversal::{Action, Named, VisResult, Visitor};
use crate::ir::RRC;
use std::rc::Rc;

pub struct InferStaticTiming<'a> {
    /// primitive name -> (go signal, done signal, latency)
    prim_latency_data: HashMap<&'a str, (&'a str, &'a str, u64)>,
}

impl Named for InferStaticTiming<'_> {
    fn name() -> &'static str {
        "infer-static-timing"
    }

    fn description() -> &'static str {
        "infers and annotates static timing for groups when possible"
    }
}

impl Default for InferStaticTiming<'_> {
    fn default() -> Self {
        let prim_latency_data = [("std_reg", ("write_en", "done", 1))]
            .iter()
            .cloned()
            .collect();
        InferStaticTiming { prim_latency_data }
    }
}

fn mem_wrt_dep_graph<'a>(
    src: &ir::Port,
    dst: &ir::Port,
    latency_data: &HashMap<&'a str, (&'a str, &'a str, u64)>,
) -> bool {

    match (&src.parent, &dst.parent) {
        (ir::PortParent::Cell(src_cell), ir::PortParent::Cell(dst_cell)) => {
            if let (
                ir::CellType::Primitive {
                    name: dst_cell_prim_type,
                    ..
                },
                ir::CellType::Primitive {
                    name: src_cell_prim_type,
                    ..
                },
            ) = (
                &dst_cell.upgrade().unwrap().borrow().prototype,
                &src_cell.upgrade().unwrap().borrow().prototype,
            ) {
                
                println!("comparing {}.{} and {}.{}", src.get_parent_name().to_string(), src.name, dst.get_parent_name().to_string(), dst.name);
                let data_dst = latency_data.get(dst_cell_prim_type.as_ref());
                let data_src = latency_data.get(src_cell_prim_type.as_ref());
                if let (Some((go_dst, done_dst, _)), Some((go_src, done_src, _))) =
                    (data_dst, data_src)
                {
                    if dst.name == *go_dst && src.name == *done_src {
                        return true;
                    }
                }
            }

            // A constant writes to a cell: to be added to the graph, the cell needs to be a "done" port.
            if let (
                ir::CellType::Primitive {
                    name: dst_cell_prim_type,
                    ..
                },
                ir::CellType::Constant { .. },
            ) = (
                &dst_cell.upgrade().unwrap().borrow().prototype,
                &src_cell.upgrade().unwrap().borrow().prototype,
            ) {
                let data = latency_data.get(dst_cell_prim_type.as_ref());
                if let Some((go, _, _)) = data {
                    if dst.name == *go {
                        return true;
                    }
                }
            }

            return false;
        }

        // Something is written to a group: to be added to the graph, this needs to be a "done" port.
        (_, ir::PortParent::Group(_)) => {
            if dst.name == "done" {
                return true;
            } else {
                return false;
            }
        }

        // If we encounter anything else, no need to add it to the graph.
        _ => return false,
    }
}

/// Attempts to infer the number of cycles starting when
/// group[go] is high, and port is high. If inference is
/// not possible, returns None.
fn infer_latency<'a>(
    group: &ir::Group,
    latency_data: &HashMap<&'a str, (&'a str, &'a str, u64)>,
    comp: &ir::Component,
) -> Option<u64> {
    let g = GraphAnalysis::from(group);
    println!("{}", g.to_string());
    let graph2 = g.edge_induced_subgraph(|src, dst| {
        mem_wrt_dep_graph(src, dst, latency_data)
    });

    let rw_set = ReadWriteSet::uses(&group.assignments);

    let mut go_done_edges: Vec<(RRC<ir::Port>, RRC<ir::Port>)> = Vec::new();
    for cell_ref in rw_set {
        let cell = cell_ref.borrow();
        if let ir::CellType::Primitive { name: cell_type, .. } = &cell.prototype {
            let (go, done, _) = latency_data.get(cell_type.as_ref()).unwrap();
            let go_port = &cell.ports.iter().find(|p| p.borrow().name == *go);
            let done_port = &cell.ports.iter().find(|p| p.borrow().name == *done);

            if let (Some(g), Some(d)) = (go_port, done_port) {
                go_done_edges.push((Rc::clone(&g), Rc::clone(&d)));
            }
        }
    }

    let graph3 = graph2.add_edges(&go_done_edges);
    println!("{}", graph3.to_string());
    let graph4 = graph3.remove_isolated_vertices();
    println!("graph:");
    println!("{}", graph4.to_string());

    let mut tsort = graph4.toposort();
    let start = tsort.next().unwrap();
    let finish = tsort.last().unwrap();
    
    println!("{}.{}", start.borrow().get_parent_name().to_string(), start.borrow().name);
    println!("{}.{}", finish.borrow().get_parent_name().to_string(), finish.borrow().name);

    let paths = graph4.paths(&*start.borrow(), &*finish.borrow());
    let path1 = paths.get(0).unwrap();
    for port in path1 {
        println!("{:?}", port);
    }

    None
}

impl Visitor<()> for InferStaticTiming<'_> {
    fn start(
        &mut self,
        comp: &mut ir::Component,
        _c: &lib::LibrarySignatures,
    ) -> VisResult<()> {
        let mut latency_result: Option<u64>;
        for group in &comp.groups {
            if let Some(latency) =
                infer_latency(&group.borrow(), &self.prim_latency_data, comp)
            {
                let grp = group.borrow();
                if let Some(curr_lat) = grp.attributes.get("static") {
                    if *curr_lat != latency {
                        return Err(Error::ImpossibleLatencyAnnotation(
                            grp.name.to_string(),
                            *curr_lat,
                            latency,
                        ));
                    }
                }
                latency_result = Some(latency);
            } else {
                latency_result = None;
            }

            match latency_result {
                Some(res) => {
                    group
                        .borrow_mut()
                        .attributes
                        .insert("static".to_string(), res);
                }
                None => continue,
            }
        }
        Ok(Action::stop_default())
    }
}