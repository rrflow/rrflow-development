use super::super::*;
use rquickjs::{CatchResultExt as _, Context as JavaScriptContext, Runtime as JavaScriptRuntime};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub(super) fn execute(
    source: &str,
    input_bytes: &[u8],
    definition: &FunctionDefinition,
) -> Result<(Vec<u8>, u32, u64)> {
    let runtime = JavaScriptRuntime::new()
        .map_err(|error| ServiceError::Function(format!("QuickJS runtime: {error}")))?;
    runtime.set_memory_limit(
        definition
            .limits
            .memory_bytes
            .try_into()
            .map_err(|_| ServiceError::FunctionLimit("memory limit exceeds usize".into()))?,
    );
    runtime.set_max_stack_size(definition.limits.stack_bytes as usize);
    let interrupts = Arc::new(AtomicU32::new(0));
    let interrupt_counter = Arc::clone(&interrupts);
    let max_interrupts = definition.limits.max_interrupts;
    runtime.set_interrupt_handler(Some(Box::new(move || {
        interrupt_counter.fetch_add(1, Ordering::Relaxed) >= max_interrupts
    })));
    let context = JavaScriptContext::full(&runtime)
        .map_err(|error| ServiceError::Function(format!("QuickJS context: {error}")))?;
    let input = std::str::from_utf8(input_bytes)
        .map_err(|error| ServiceError::Function(format!("input is not UTF-8: {error}")))?;
    let encoded_input = serde_json::to_string(input).map_err(contract_json)?;
    let script = format!(
        r#""use strict";
Object.defineProperty(globalThis, "Date", {{ value: undefined, writable: false, configurable: false }});
Object.defineProperty(globalThis, "Intl", {{ value: undefined, writable: false, configurable: false }});
Object.defineProperty(globalThis, "eval", {{ value: undefined, writable: false, configurable: false }});
Object.defineProperty(globalThis, "Function", {{ value: undefined, writable: false, configurable: false }});
Object.defineProperty(Math, "random", {{ value: undefined, writable: false, configurable: false }});
const __rrd_input = JSON.parse({encoded_input});
const __rrd_function = ({source});
if (typeof __rrd_function !== "function") throw new TypeError("rrd function source must evaluate to a function");
const __rrd_output = __rrd_function(__rrd_input);
if (__rrd_output && typeof __rrd_output.then === "function") throw new TypeError("rrd-function-v1 is synchronous");
const __rrd_json = JSON.stringify(__rrd_output);
if (typeof __rrd_json !== "string") throw new TypeError("rrd function output must be JSON serializable");
__rrd_json;"#
    );
    let output = context
        .with(|context| {
            context
                .eval::<String, _>(script.as_bytes())
                .catch(&context)
                .map_err(|error| error.to_string())
        })
        .map_err(|message| {
            if interrupts.load(Ordering::Relaxed) > max_interrupts {
                ServiceError::FunctionLimit("JavaScript interrupt budget exhausted".into())
            } else if message.to_ascii_lowercase().contains("memory")
                || message.to_ascii_lowercase().contains("stack")
            {
                ServiceError::FunctionLimit(format!("JavaScript resource limit: {message}"))
            } else {
                ServiceError::Function(format!("JavaScript execution failed: {message}"))
            }
        })?;
    Ok((output.into_bytes(), interrupts.load(Ordering::Relaxed), 0))
}
