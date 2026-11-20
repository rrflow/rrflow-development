//! The Vortex reference module.

use super::{failed, from_batches, to_batch};
use crate::columnar::{ColumnarFormat, ColumnarTable};
use crate::Result;
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use std::sync::Arc;
use vortex::array::expr::{root, select};
use vortex::arrow::convert::from_arrow_batch;
use vortex::file::{OpenOptionsSessionExt, VortexFile, WriteOptionsSessionExt};
use vortex::io::runtime::single::SingleThreadRuntime;
use vortex::io::runtime::BlockingRuntime;
use vortex::io::session::RuntimeSessionExt;
use vortex::session::VortexSession;
use vortex::VortexSessionDefault;

const MODULE: &str = "vortex";

/// The Vortex file format behind the columnar port, with its default write
/// strategy and compressor. Each call runs on its own single-thread runtime,
/// so the module needs no async runtime from its caller.
#[derive(Debug, Default, Clone, Copy)]
pub struct VortexReference;

impl ColumnarFormat for VortexReference {
    fn id(&self) -> &'static str {
        MODULE
    }

    fn encode(&self, table: &ColumnarTable) -> Result<Vec<u8>> {
        let runtime = SingleThreadRuntime::default();
        let session = VortexSession::default().with_handle(runtime.handle());
        let array =
            from_arrow_batch(&to_batch(table)?, false).map_err(|error| failed(MODULE, error))?;
        let mut artifact = Vec::new();
        session
            .write_options()
            .blocking(&runtime)
            .write(&mut artifact, array.to_array_iterator())
            .map_err(|error| failed(MODULE, error))?;
        Ok(artifact)
    }

    fn decode(&self, artifact: &[u8]) -> Result<ColumnarTable> {
        read(artifact, None)
    }

    fn decode_columns(&self, artifact: &[u8], names: &[&str]) -> Result<ColumnarTable> {
        read(artifact, Some(names))
    }
}

fn read(artifact: &[u8], names: Option<&[&str]>) -> Result<ColumnarTable> {
    let runtime = SingleThreadRuntime::default();
    let session = VortexSession::default().with_handle(runtime.handle());
    let file = session
        .open_options()
        .open_buffer(artifact.to_vec())
        .map_err(|error| failed(MODULE, error))?;
    let stored = stored_schema(&file)?;
    let mut scan = file.scan().map_err(|error| failed(MODULE, error))?;
    let schema: SchemaRef = match names {
        None => stored,
        Some(names) => {
            let fields = names
                .iter()
                .map(|name| {
                    stored
                        .field_with_name(name)
                        .cloned()
                        .map_err(|_| failed(MODULE, format!("no column named {name}")))
                })
                .collect::<Result<Vec<_>>>()?;
            let projection = select(names.to_vec(), root())
                .bind(file.dtype())
                .map_err(|error| failed(MODULE, error))?;
            scan = scan.with_projection(projection);
            Arc::new(Schema::new(fields))
        }
    };
    let batches = scan
        .into_record_batch_reader(Arc::clone(&schema), &runtime)
        .map_err(|error| failed(MODULE, error))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| failed(MODULE, error))?;
    from_batches(&schema, &batches)
}

/// The Arrow schema of the stored columns, read from the file's own data type:
/// the port's two column types, every column nullable.
fn stored_schema(file: &VortexFile) -> Result<SchemaRef> {
    let fields = file
        .dtype()
        .as_struct_fields_opt()
        .ok_or_else(|| failed(MODULE, "the file does not hold a table"))?;
    let columns = fields
        .names()
        .iter()
        .zip(fields.fields())
        .map(|(name, dtype)| {
            let data_type = if dtype.is_utf8() {
                DataType::Utf8
            } else if dtype.is_unsigned_int() {
                DataType::UInt64
            } else {
                return Err(failed(
                    MODULE,
                    format!("column {name} has unsupported type {dtype}"),
                ));
            };
            Ok(Field::new(name.as_ref(), data_type, true))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Arc::new(Schema::new(columns)))
}
