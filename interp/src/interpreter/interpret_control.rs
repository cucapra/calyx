//! Inteprets a control in a component.

use super::interpret_group::{
    finish_group_interpretation, interp_cont, interpret_group,
};
use crate::environment::InterpreterState;
use calyx::{errors::FutilResult, ir};

/// Helper function to evaluate control
pub fn interpret_control(
    ctrl: &ir::Control,
    continuous_assignments: &[ir::Assignment],
    env: InterpreterState,
    comp: &ir::Component,
) -> FutilResult<InterpreterState> {
    match ctrl {
        ir::Control::Seq(s) => eval_seq(s, continuous_assignments, env, comp),
        ir::Control::Par(p) => eval_par(p, continuous_assignments, env),
        ir::Control::If(i) => eval_if(i, continuous_assignments, env, comp),
        ir::Control::While(w) => {
            eval_while(w, continuous_assignments, env, comp)
        }
        ir::Control::Invoke(i) => eval_invoke(i, continuous_assignments, env),
        ir::Control::Enable(e) => eval_enable(e, continuous_assignments, env),
        ir::Control::Empty(e) => {
            eval_empty(e, continuous_assignments, env, comp)
        }
    }
}

/// Interpret Seq
fn eval_seq(
    s: &ir::Seq,
    continuous_assignments: &[ir::Assignment],
    mut env: InterpreterState,
    comp: &ir::Component,
) -> FutilResult<InterpreterState> {
    for stmt in &s.stmts {
        env = interpret_control(stmt, continuous_assignments, env, comp)?;
    }
    Ok(env)
}

/// Interpret Par
/// at the moment behaves like seq
fn eval_par(
    _p: &ir::Par,
    _continuous_assignments: &[ir::Assignment],
    mut _env: InterpreterState,
) -> FutilResult<InterpreterState> {
    // for stmt in &p.stmts {
    //     env = interpret_control(stmt, comp.clone(), env)?;
    // }
    todo!("par control operator")
}

/// Interpret If
fn eval_if(
    i: &ir::If,
    continuous_assignments: &[ir::Assignment],
    mut env: InterpreterState,
    comp: &ir::Component,
) -> FutilResult<InterpreterState> {
    env = interpret_group(&i.cond.borrow(), continuous_assignments, env)?;
    let cond_flag = env.get_from_port(&i.port.borrow()).as_u64();
    env = finish_group_interpretation(
        &i.cond.borrow(),
        continuous_assignments,
        env,
    )?;

    let target = if cond_flag == 0 {
        &i.fbranch
    } else {
        &i.tbranch
    };

    interpret_control(target, continuous_assignments, env, comp)
}

/// Interpret While
// /// The loop statement is similar to the conditional. It enables
// cond_group and uses port_name as the conditional value. When the
// value is high, it executes body_stmt and recomputes the conditional
// using cond_group.
fn eval_while(
    w: &ir::While,
    continuous_assignments: &[ir::Assignment],
    mut env: InterpreterState,
    comp: &ir::Component,
) -> FutilResult<InterpreterState> {
    loop {
        env = interpret_group(&w.cond.borrow(), continuous_assignments, env)?;

        let cond_val = env.get_from_port(&w.port.borrow()).as_u64();
        env = finish_group_interpretation(
            &w.cond.borrow(),
            continuous_assignments,
            env,
        )?;

        if cond_val == 0 {
            break;
        }

        env = interpret_control(&w.body, continuous_assignments, env, comp)?;
    }

    Ok(env)
}

/// Interpret Invoke
/// TODO
#[allow(clippy::unnecessary_wraps)]
fn eval_invoke(
    _i: &ir::Invoke,
    _continuous_assignments: &[ir::Assignment],
    _env: InterpreterState,
) -> FutilResult<InterpreterState> {
    todo!("invoke control operator")
}

/// Interpret Enable
fn eval_enable(
    e: &ir::Enable,
    continuous_assignments: &[ir::Assignment],
    mut env: InterpreterState,
) -> FutilResult<InterpreterState> {
    env = interpret_group(&e.group.borrow(), continuous_assignments, env)?;
    finish_group_interpretation(&e.group.borrow(), continuous_assignments, env)
}

/// Interpret Empty
#[allow(clippy::unnecessary_wraps)]
fn eval_empty(
    _e: &ir::Empty,
    continuous_assignments: &[ir::Assignment],
    mut env: InterpreterState,
    comp: &ir::Component,
) -> FutilResult<InterpreterState> {
    env = interp_cont(continuous_assignments, env, comp)?;
    Ok(env)
}
