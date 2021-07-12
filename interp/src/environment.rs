//! Environment for interpreter.

use super::stk_env::Smoosher;
use super::utils::MemoryMap;
use super::{primitives, primitives::Primitive, values::Value};
use calyx::ir::{self, RRC};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

/// A raw pointer reference to a cell. Can only be used as a key, but cannot be
/// used to access the cell itself
type ConstCell = *const ir::Cell;

/// A raw pointer reference to a port. As with cell, it is only suitable for use
/// as a key and cannot be used to access the port itself
type ConstPort = *const ir::Port;

/// A map defining primitive implementations for Cells. As it is keyed by
/// CellRefs the lifetime of the keys is independent of the actual cells
type PrimitiveMap = RRC<HashMap<ConstCell, primitives::Primitive>>;

/// A map defining values for ports. As it is keyed by PortRefs, the lifetime of
/// the keys is independent of the ports. However as a result it is flat, rather
/// than heirarchical which simplifies the access interface
type PortValMap = Smoosher<ConstPort, Value>;

/// The environment to interpret a Calyx program.
#[derive(Debug)]
pub struct InterpreterState {
    ///clock count
    pub clk: u64,

    ///mapping from cells to prims
    pub cell_prim_map: PrimitiveMap,

    ///use raw pointers for hashmap: ports to values
    //this is a Smoosher (see stk_env.rs)
    pub pv_map: PortValMap,

    /// A reference to the context.
    pub context: ir::RRC<ir::Context>,
}

/// Helper functions for the environment.
impl InterpreterState {
    /// Construct an environment
    /// ctx : A context from the IR
    pub fn init(ctx: &ir::RRC<ir::Context>, mems: &Option<MemoryMap>) -> Self {
        Self {
            context: ctx.clone(),
            clk: 0,
            pv_map: InterpreterState::construct_pv_map(&ctx.borrow()),
            cell_prim_map: InterpreterState::construct_cp_map(
                &ctx.borrow(),
                mems,
            ),
        }
    }

    pub fn insert(&mut self, port: ConstPort, value: Value) {
        self.pv_map.set(port, value);
    }

    //all of these use parameters as values for constuctors
    fn construct_cp_map(
        ctx: &ir::Context,
        mems: &Option<MemoryMap>,
    ) -> PrimitiveMap {
        let mut map = HashMap::new();
        for comp in &ctx.components {
            for cell in comp.cells.iter() {
                let cl: &ir::Cell = &cell.borrow();
                let cell_name = cl.name();

                if let ir::CellType::Primitive { name, .. } = &cl.prototype {
                    match name.as_ref() {
                        "std_reg" => {
                            let reg = primitives::StdReg::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdReg(reg));
                        }
                        "std_const" => {
                            let width = cl.get_parameter("WIDTH").unwrap();
                            let cst = primitives::StdConst::new(
                                width,
                                Value::try_from_init(
                                    cl.get_parameter("VALUE").unwrap(),
                                    width,
                                )
                                .unwrap(),
                            );
                            map.insert(
                                cl as ConstCell,
                                Primitive::StdConst(cst),
                            );
                        }
                        "std_lsh" => {
                            let width = cl.get_parameter("WIDTH").unwrap();
                            let lsh = primitives::StdLsh::new(width);
                            map.insert(cl as ConstCell, Primitive::StdLsh(lsh));
                        }
                        "std_rsh" => {
                            let width = cl.get_parameter("WIDTH").unwrap();
                            let rsh = primitives::StdRsh::new(width);
                            map.insert(cl as ConstCell, Primitive::StdRsh(rsh));
                        }
                        "std_add" => {
                            let adder = primitives::StdAdd::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(
                                cl as ConstCell,
                                Primitive::StdAdd(adder),
                            );
                        }
                        "std_sub" => {
                            let width = cl.get_parameter("WIDTH").unwrap();
                            let sub = primitives::StdSub::new(width);
                            map.insert(cl as ConstCell, Primitive::StdSub(sub));
                        }
                        "std_slice" => {
                            let slc = primitives::StdSlice::new(
                                cl.get_parameter("IN_WIDTH").unwrap(),
                                cl.get_parameter("OUT_WIDTH").unwrap(),
                            );
                            map.insert(
                                cl as ConstCell,
                                Primitive::StdSlice(slc),
                            );
                        }
                        "std_pad" => {
                            let pad = primitives::StdPad::new(
                                cl.get_parameter("IN_WIDTH").unwrap(),
                                cl.get_parameter("OUT_WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdPad(pad));
                        }
                        "std_not" => {
                            let not = primitives::StdNot::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdNot(not));
                        }
                        "std_and" => {
                            let and = primitives::StdAnd::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdAnd(and));
                        }
                        "std_or" => {
                            let or = primitives::StdOr::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdOr(or));
                        }
                        "std_xor" => {
                            let xor = primitives::StdXor::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdXor(xor));
                        }
                        "std_ge" => {
                            let ge = primitives::StdGe::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdGe(ge));
                        }
                        "std_gt" => {
                            let gt = primitives::StdGt::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdGt(gt));
                        }
                        "std_eq" => {
                            let eq = primitives::StdEq::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdEq(eq));
                        }
                        "std_neq" => {
                            let neq = primitives::StdNeq::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdNeq(neq));
                        }
                        "std_le" => {
                            let le = primitives::StdLe::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdLe(le));
                        }
                        "std_lt" => {
                            let lt = primitives::StdLt::new(
                                cl.get_parameter("WIDTH").unwrap(),
                            );
                            map.insert(cl as ConstCell, Primitive::StdLt(lt));
                        }
                        "std_mem_d1" => {
                            let mut m1 = primitives::StdMemD1::new(
                                cl.get_parameter("WIDTH").unwrap(),
                                cl.get_parameter("SIZE").unwrap(),
                                cl.get_parameter("IDX_SIZE").unwrap(),
                            );
                            if let Some(v) = mems
                                .as_ref()
                                .map(|x| x.get(cell_name))
                                .flatten()
                            {
                                m1.initialize_memory(v)
                            }

                            map.insert(
                                cl as ConstCell,
                                Primitive::StdMemD1(m1),
                            );
                        }
                        "std_mem_d2" => {
                            let mut m2 = primitives::StdMemD2::new(
                                cl.get_parameter("WIDTH").unwrap(),
                                cl.get_parameter("D0_SIZE").unwrap(),
                                cl.get_parameter("D1_SIZE").unwrap(),
                                cl.get_parameter("D0_IDX_SIZE").unwrap(),
                                cl.get_parameter("D1_IDX_SIZE").unwrap(),
                            );

                            if let Some(v) = mems
                                .as_ref()
                                .map(|x| x.get(cell_name))
                                .flatten()
                            {
                                m2.initialize_memory(v)
                            }

                            map.insert(
                                cl as ConstCell,
                                Primitive::StdMemD2(m2),
                            );
                        }
                        "std_mem_d3" => {
                            let mut m3 = primitives::StdMemD3::new(
                                cl.get_parameter("WIDTH").unwrap(),
                                cl.get_parameter("D0_SIZE").unwrap(),
                                cl.get_parameter("D1_SIZE").unwrap(),
                                cl.get_parameter("D2_SIZE").unwrap(),
                                cl.get_parameter("D0_IDX_SIZE").unwrap(),
                                cl.get_parameter("D1_IDX_SIZE").unwrap(),
                                cl.get_parameter("D2_IDX_SIZE").unwrap(),
                            );

                            if let Some(v) = mems
                                .as_ref()
                                .map(|x| x.get(cell_name))
                                .flatten()
                            {
                                m3.initialize_memory(v)
                            }

                            map.insert(
                                cl as ConstCell,
                                Primitive::StdMemD3(m3),
                            );
                        }
                        "std_mem_d4" => {
                            let mut m4 = primitives::StdMemD4::new(
                                cl.get_parameter("WIDTH").unwrap(),
                                cl.get_parameter("D0_SIZE").unwrap(),
                                cl.get_parameter("D1_SIZE").unwrap(),
                                cl.get_parameter("D2_SIZE").unwrap(),
                                cl.get_parameter("D3_SIZE").unwrap(),
                                cl.get_parameter("D0_IDX_SIZE").unwrap(),
                                cl.get_parameter("D1_IDX_SIZE").unwrap(),
                                cl.get_parameter("D2_IDX_SIZE").unwrap(),
                                cl.get_parameter("D3_IDX_SIZE").unwrap(),
                            );

                            if let Some(v) = mems
                                .as_ref()
                                .map(|x| x.get(cell_name))
                                .flatten()
                            {
                                m4.initialize_memory(v)
                            }

                            map.insert(
                                cl as ConstCell,
                                Primitive::StdMemD4(m4),
                            );
                        }
                        e => panic!("Unknown primitive {}", e),
                    }
                }
            }
        }
        Rc::new(RefCell::new(map))
    }

    fn construct_pv_map(ctx: &ir::Context) -> PortValMap {
        let mut map = HashMap::new();
        for comp in &ctx.components {
            for port in comp.signature.borrow().ports.iter() {
                let pt: &ir::Port = &port.borrow();
                map.insert(
                    pt as *const ir::Port,
                    Value::try_from_init(0, 1).unwrap(),
                );
            }
            for group in comp.groups.iter() {
                let grp = group.borrow();
                for hole in &grp.holes {
                    let pt: &ir::Port = &hole.borrow();
                    map.insert(
                        pt as ConstPort,
                        Value::try_from_init(0, 1).unwrap(),
                    );
                }
            }
            for cell in comp.cells.iter() {
                //also iterate over groups cuz they also have ports
                //iterate over ports, getting their value and putting into map
                let cll = cell.borrow();
                match &cll.prototype {
                    ir::CellType::Constant { val, width } => {
                        for port in &cll.ports {
                            let pt: &ir::Port = &port.borrow();
                            map.insert(
                                pt as ConstPort,
                                Value::try_from_init(*val, *width).unwrap(),
                            );
                        }
                    }
                    ir::CellType::Primitive { .. } => {
                        for port in &cll.ports {
                            let pt: &ir::Port = &port.borrow();
                            map.insert(
                                pt as ConstPort,
                                Value::try_from_init(
                                    cll.get_parameter("VALUE")
                                        .unwrap_or_default(),
                                    pt.width,
                                )
                                .unwrap(),
                            );
                        }
                    }
                    ir::CellType::Component { .. } => {
                        for port in &cll.ports {
                            let pt: &ir::Port = &port.borrow();
                            map.insert(
                                pt as ConstPort,
                                Value::try_from_init(0, 0).unwrap(),
                            );
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        map.into()
    }

    /// Return the value associated with a component's port.
    pub fn get_from_port(&self, port: &ir::Port) -> &Value {
        &self.pv_map.get(&(port as ConstPort)).unwrap()
    }

    pub fn get_from_const_port(&self, port: *const ir::Port) -> &Value {
        &self.pv_map.get(&port).unwrap()
    }

    /// Gets the cell in a component based on the name;
    /// XXX: similar to find_cell in component.rs
    /// Does this function *need* to be in environment?
    pub fn get_cell(
        &self,
        comp: &ir::Id,
        cell: &ir::Id,
    ) -> Option<ir::RRC<ir::Cell>> {
        let a = self.context.borrow();
        let temp = a.components.iter().find(|cm| cm.name == *comp)?;
        temp.find_cell(&(cell.id))
    }

    /// Outputs the cell state;
    ///TODO (write to a specified output in the future) We could do the printing
    ///of values here for tracing purposes as discussed. Could also have a
    ///separate DS that we could put the cell states into for more custom tracing
    pub fn print_env(&self) {
        println!("{}", serde_json::to_string_pretty(&self).unwrap());
    }

    pub fn cell_is_comb(&self, cell: &ir::Cell) -> bool {
        self.cell_prim_map
            .borrow()
            .get(&(cell as ConstCell))
            .unwrap()
            .is_comb()
    }

    pub fn fork(&mut self) -> Self {
        let other_pv_map = if self.pv_map.top().is_empty() {
            self.pv_map.fork_from_tail()
        } else {
            self.pv_map.fork()
        };
        Self {
            clk: self.clk,
            cell_prim_map: Rc::clone(&self.cell_prim_map),
            pv_map: other_pv_map,
            context: Rc::clone(&self.context),
        }
    }
}

impl Serialize for InterpreterState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ctx: &ir::Context = &self.context.borrow();

        let cell_prim_map = self.cell_prim_map.borrow();

        let bmap: BTreeMap<_, _> = ctx
            .components
            .iter()
            .map(|comp| {
                let inner_map: BTreeMap<_, _> = comp
                    .cells
                    .iter()
                    .map(|cell| {
                        let inner_map: BTreeMap<_, _> = cell
                            .borrow()
                            .ports
                            .iter()
                            .map(|port| {
                                (
                                    port.borrow().name.clone(),
                                    self.get_from_port(&port.borrow()).as_u64(),
                                )
                            })
                            .collect();
                        (cell.borrow().name().clone(), inner_map)
                    })
                    .collect();
                (comp.name.clone(), inner_map)
            })
            .collect();

        let cell_map: BTreeMap<_, _> = ctx
            .components
            .iter()
            .map(|comp| {
                let inner_map: BTreeMap<_, _> = comp
                    .cells
                    .iter()
                    .filter_map(|cell| {
                        if let Some(prim) = cell_prim_map
                            .get(&(&cell.borrow() as &ir::Cell as ConstCell))
                        {
                            if !prim.is_comb() {
                                return Some((
                                    cell.borrow().name().clone(),
                                    prim,
                                ));
                            }
                        }
                        None
                    })
                    .collect();
                (comp.name.clone(), inner_map)
            })
            .collect();

        let p = Printable {
            ports: bmap,
            memories: cell_map,
        };
        p.serialize(serializer)
    }
}

#[derive(Serialize)]
struct Printable<'a> {
    ports: BTreeMap<ir::Id, BTreeMap<ir::Id, BTreeMap<ir::Id, u64>>>,
    memories: BTreeMap<ir::Id, BTreeMap<ir::Id, &'a Primitive>>,
}
