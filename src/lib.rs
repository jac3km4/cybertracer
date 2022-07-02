#![feature(thread_id_value)]
use std::fs::OpenOptions;
use std::io::Write;
use std::mem;
use std::thread::ThreadId;

use ansi_term::ANSIGenericStrings;
use dashmap::DashMap;
use detour::static_detour;
use frame::{StackFrameInfo, StackTrace};
use natives::StackFrame;
use stybulate::{Cell, Headers, Style, Table};
use time::{format_description, OffsetDateTime};

mod frame;
mod natives;

static_detour! {
  static CallFunc: extern "C" fn(usize, *mut StackFrame, usize, usize);
  static CrashFunc: extern "C" fn(u8, usize) -> u32;
//   static CrashFunc2: extern "C" fn(usize, usize, usize) -> u8; // 0x2B3D640
}

pub const TRACE_FILE: &str = "trace.log";

#[ctor::ctor]
static STACK_TRACES: DashMap<ThreadId, StackTrace> = DashMap::new();

#[ctor::ctor]
fn main() {
    if let Err(err) = unsafe { install_hooks() } {
        write_trace(format!("failed to set up hooks: {err}")).unwrap();
    }
}

#[ctor::dtor]
fn exit() {
    dump_traces();
}

unsafe fn install_hooks() -> Result<(), detour::Error> {
    CallFunc.initialize(
        mem::transmute(memhack::resolve_rva(natives::CALL_FUNC_RVA)),
        script_call_wrapper,
    )?;
    CallFunc.enable()?;

    CrashFunc.initialize(
        mem::transmute(memhack::resolve_rva(natives::CRASH_FUNC_RVA)),
        crash_wrapper,
    )?;
    CrashFunc.enable()?;

    Ok(())
}

fn script_call_wrapper(arg1: usize, frame: *mut StackFrame, arg3: usize, arg4: usize) {
    let mut trace = StackTrace::default();
    let mut cur = frame;

    while !cur.is_null() {
        let frame = unsafe { &*cur };
        let func = unsafe { &*frame.func };
        let class = unsafe { ((*func.vft).get_class)(func) };
        let info = StackFrameInfo {
            function: Some(func.name),
            class: class.map(|c| c.name),
        };
        if trace.try_push(info).is_err() {
            break;
        }

        cur = frame.parent;
    }
    STACK_TRACES.insert(std::thread::current().id(), trace);

    CallFunc.call(arg1, frame, arg3, arg4)
}

fn crash_wrapper(arg1: u8, arg2: usize) -> u32 {
    dump_traces();
    CrashFunc.call(arg1, arg2)
}

fn dump_traces() {
    let mut trace_data = vec![];

    for trace in STACK_TRACES.iter() {
        let mut trace_fragments = vec![];
        for (i, frame) in trace.iter().rev().enumerate() {
            match frame {
                StackFrameInfo {
                    function: Some(fun),
                    class: Some(class),
                } => {
                    trace_fragments.push(class.resolve().into());
                    trace_fragments.push("::".into());
                    trace_fragments.push(fun.resolve().split(';').next().unwrap().into());
                }
                StackFrameInfo {
                    function: Some(fun),
                    class: None,
                } => {
                    trace_fragments.push(fun.resolve().split(';').next().unwrap().into());
                }
                _ => {}
            }
            if i != trace.size() - 1 {
                trace_fragments.push("\n ↳ ".into());
            }
        }
        trace_data.push((trace.key().as_u64(), trace_fragments));
    }

    trace_data.sort_by_key(|&(thread, _)| thread);

    let mut rows = vec![];
    for (thread, strings) in &trace_data {
        let row = vec![
            Cell::Int(thread.get() as i32),
            Cell::Text(Box::new(ANSIGenericStrings(strings))),
        ];
        rows.push(row);
    }

    let output = Table::new(
        Style::Github,
        rows,
        Some(Headers::from(vec!["thread", "stack trace"])),
    )
    .tabulate();

    write_trace(output).unwrap()
}

fn write_trace(message: impl AsRef<str>) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().append(true).create(true).open(TRACE_FILE)?;
    let time = OffsetDateTime::now_local()?.format(&format_description::well_known::Rfc2822)?;

    writeln!(file)?;
    writeln!(file, "Game crashed at {time}")?;
    writeln!(file, "Traces for all game threads:")?;
    writeln!(file, "{}", message.as_ref())?;
    writeln!(file)?;
    Ok(())
}
