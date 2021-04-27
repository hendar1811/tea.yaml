use crate::ast::{Ast, BinOp, Env, Program, Val};
use im::HashMap;
use std::error;
use std::fmt;

#[derive(PartialEq, Debug)]
pub struct InterpError(pub String);
impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl error::Error for InterpError {}

type FuncEnv<'a> = HashMap<&'a String, Val<'a>>;

pub fn interpret(program: &Program) -> Result<Vec<Val>, InterpError> {
    let funcs = find_functions(program)?;
    let mut env = HashMap::new();
    let mut vals = vec![];

    for expr in program {
        match interpret_top_level(expr, env.clone(), &funcs)? {
            ValOrEnv::V(val) => vals.push(val),
            ValOrEnv::E(new_env) => env = new_env,
        }
    }

    Ok(vals)
}

fn find_functions(program: &Program) -> Result<FuncEnv, InterpError> {
    let mut env = HashMap::new();

    for expr in program {
        match expr {
            Ast::FunctionNode(name, params, body) => {
                env.insert(name, Val::Lam(params, body, HashMap::new()));
            }
            _ => (),
        };
    }

    return Ok(env);
}

enum ValOrEnv<'a> {
    V(Val<'a>),
    E(Env<'a>),
}

fn interpret_top_level<'a>(
    expr: &'a Ast,
    env: Env<'a>,
    func_table: &FuncEnv<'a>,
) -> Result<ValOrEnv<'a>, InterpError> {
    match expr {
        Ast::LetNodeTopLevel(id, binding) => {
            let val = interpret_env(binding, env.clone(), func_table)?;
            Ok(ValOrEnv::E(env.update(id.clone(), val)))
        }
        Ast::LetNode(_, _, _) => Err(InterpError(
            "Found LetNode instead of LetNodeToplevel on top level".to_string(),
        )),
        Ast::FunctionNode(_, _, _) => Ok(ValOrEnv::E(env)),
        e => Ok(ValOrEnv::V(interpret_env(e, env, func_table)?)),
    }
}

fn interpret_env<'a>(
    expr: &'a Ast,
    env: Env<'a>,
    func_table: &FuncEnv<'a>,
) -> Result<Val<'a>, InterpError> {
    match expr {
        Ast::NumberNode(n) => Ok(Val::Num(n.clone())),
        Ast::BoolNode(v) => Ok(Val::Bool(v.clone())),
        Ast::VarNode(id) => match env.get(id) {
            Some(v) => Ok(v.clone()),
            None => match func_table.get(id) {
                Some(v) => Ok(v.clone()),
                None => Err(InterpError(
                    format!("Couldn't find var in environment: {}", id).to_string(),
                )),
            },
        },
        Ast::LetNode(id, binding, body) => interpret_env(
            body,
            env.update(id.clone(), interpret_env(binding, env.clone(), func_table)?),
            func_table,
        ),
        Ast::LetNodeTopLevel(_, _) => Err(InterpError(
            "Found LetNodeTopLevel instead of LetNode in expression".to_string(),
        )),
        Ast::BinOpNode(op, e1, e2) => interpret_binop(*op, e1, e2, env, func_table),
        Ast::LambdaNode(params, body) => Ok(Val::Lam(params, body, env.clone())),
        Ast::FunCallNode(fun, args) => {
            let fun_value = interpret_env(fun, env.clone(), func_table)?;
            match fun_value {
                Val::Lam(params, body, lam_env) => {
                    let mut new_env: Env = HashMap::new();
                    for (param, arg) in params.iter().zip(args) {
                        new_env.insert(
                            param.to_string(),
                            interpret_env(arg, env.clone(), func_table)?,
                        );
                    }
                    let mut lam_env = lam_env.clone();
                    lam_env.extend(new_env);
                    interpret_env(body, lam_env, func_table)
                }
                _ => Err(InterpError(
                    "Function call with non-function value".to_string(),
                )),
            }
        }
        Ast::IfNode(cond_e, consq_e, altern_e) => {
            match interpret_env(cond_e, env.clone(), func_table)? {
                Val::Bool(true) => interpret_env(consq_e, env.clone(), func_table),
                Val::Bool(false) => interpret_env(altern_e, env.clone(), func_table),
                _ => Err(InterpError(
                    "Conditional expression with non-boolean expression".to_string(),
                )),
            }
        }
        Ast::FunctionNode(_, _, _) => {
            Err(InterpError("Function node not at top level".to_string()))
        }
    }
}

fn interpret_binop<'a>(
    op: BinOp,
    e1: &'a Ast,
    e2: &'a Ast,
    env: Env<'a>,
    func_table: &FuncEnv<'a>,
) -> Result<Val<'a>, InterpError> {
    let op_lam = match op {
        BinOp::Plus => |x, y| match (x, y) {
            (Val::Num(xv), Val::Num(yv)) => Some(Val::Num(xv + yv)),
            _ => None,
        },
        BinOp::Minus => |x, y| match (x, y) {
            (Val::Num(xv), Val::Num(yv)) => Some(Val::Num(xv - yv)),
            _ => None,
        },
        BinOp::Times => |x, y| match (x, y) {
            (Val::Num(xv), Val::Num(yv)) => Some(Val::Num(xv * yv)),
            _ => None,
        },
        BinOp::Eq => |x, y| Some(Val::Bool(x == y)),
    };

    let v1 = interpret_env(e1, env.clone(), func_table)?;
    let v2 = interpret_env(e2, env.clone(), func_table)?;

    match op_lam(v1, v2) {
        Some(r) => Ok(r),
        None => Err(InterpError("Got incorrect types to binop".to_string())),
    }
}
