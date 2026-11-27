use super::super::*;

pub(super) fn execute(
    module_bytes: &[u8],
    input_bytes: &[u8],
    definition: &FunctionDefinition,
) -> Result<(Vec<u8>, u32, u64)> {
    let mut config = wasmi::Config::default();
    config.consume_fuel(true);
    config.compilation_mode(wasmi::CompilationMode::Eager);
    config.set_max_stack_height(definition.limits.stack_bytes as usize);
    config.set_max_recursion_depth((definition.limits.stack_bytes as usize / 1024).max(1));
    config.set_max_cached_stacks(0);
    let engine = wasmi::Engine::new(&config);
    let module = wasmi::Module::new(&engine, module_bytes)
        .map_err(|error| ServiceError::Function(format!("WebAssembly module: {error}")))?;
    if module.imports().next().is_some() {
        return Err(ServiceError::Function(
            "WebAssembly imports are forbidden by the function-v1 capability sandbox".into(),
        ));
    }
    let limits =
        wasmi::StoreLimitsBuilder::new()
            .memory_size(
                definition.limits.memory_bytes.try_into().map_err(|_| {
                    ServiceError::FunctionLimit("memory limit exceeds usize".into())
                })?,
            )
            .instances(1)
            .memories(1)
            .tables(0)
            .trap_on_grow_failure(true)
            .build();
    let mut store = wasmi::Store::new(&engine, limits);
    store.limiter(|limits| limits);
    store
        .set_fuel(definition.limits.max_fuel)
        .map_err(|error| ServiceError::Function(format!("WebAssembly fuel: {error}")))?;
    let linker = wasmi::Linker::new(&engine);
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(|error| ServiceError::Function(format!("WebAssembly instance: {error}")))?;
    let memory = instance
        .get_memory(&store, "memory")
        .ok_or_else(|| ServiceError::Function("WebAssembly memory export is missing".into()))?;
    let allocate = instance
        .get_typed_func::<i32, i32>(&store, "rrd_alloc")
        .map_err(|error| ServiceError::Function(format!("WebAssembly rrd_alloc ABI: {error}")))?;
    let run = instance
        .get_typed_func::<(i32, i32), i64>(&store, "rrd_run")
        .map_err(|error| ServiceError::Function(format!("WebAssembly rrd_run ABI: {error}")))?;
    let input_len: i32 = input_bytes
        .len()
        .try_into()
        .map_err(|_| ServiceError::FunctionLimit("WebAssembly input exceeds i32".into()))?;
    let input_ptr = allocate
        .call(&mut store, input_len)
        .map_err(map_execution_error)?;
    let input_offset: usize = input_ptr.try_into().map_err(|_| {
        ServiceError::Function("WebAssembly returned a negative input pointer".into())
    })?;
    memory
        .write(&mut store, input_offset, input_bytes)
        .map_err(|error| ServiceError::Function(format!("WebAssembly input memory: {error}")))?;
    let packed = run
        .call(&mut store, (input_ptr, input_len))
        .map_err(map_execution_error)? as u64;
    let output_ptr = (packed >> 32) as u32 as usize;
    let output_len = (packed & u64::from(u32::MAX)) as u32 as usize;
    if output_len > definition.limits.max_output_bytes as usize {
        return Err(ServiceError::FunctionLimit(
            "WebAssembly output exceeds its configured byte limit".into(),
        ));
    }
    let mut output = vec![0_u8; output_len];
    memory
        .read(&store, output_ptr, &mut output)
        .map_err(|error| ServiceError::Function(format!("WebAssembly output memory: {error}")))?;
    let fuel_remaining = store
        .get_fuel()
        .map_err(|error| ServiceError::Function(format!("WebAssembly fuel: {error}")))?;
    Ok((
        output,
        0,
        definition.limits.max_fuel.saturating_sub(fuel_remaining),
    ))
}

fn map_execution_error(error: wasmi::Error) -> ServiceError {
    let message = error.to_string();
    if message.to_ascii_lowercase().contains("fuel")
        || message.to_ascii_lowercase().contains("resource limit")
        || message.to_ascii_lowercase().contains("grow")
        || message.to_ascii_lowercase().contains("stack")
        || message.to_ascii_lowercase().contains("recursion")
    {
        ServiceError::FunctionLimit(format!("WebAssembly resource limit: {message}"))
    } else {
        ServiceError::Function(format!("WebAssembly execution failed: {message}"))
    }
}
