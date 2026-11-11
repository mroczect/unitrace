use alloc::sync::Arc;

use crate::error::Error;
use crate::schema::Schema;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    id: i64,
    fields: Vec<String>,
    schema: Arc<Schema>,
}

impl Record {
    pub fn try_new(id: i64, fields: Vec<String>, schema: Arc<Schema>) -> Result<Self, Error> {
        schema.validate_field_types(&fields)?;
        if let Some(idx) = schema.identity_index() {
            let column_value = fields
                .get(idx)
                .ok_or(Error::RecordLengthMismatch {
                    expected: schema.len(),
                    found: fields.len(),
                })?
                .clone();
            if column_value != id.to_string() {
                return Err(Error::IdentityMismatch { id, column_value });
            }
        }
        Ok(Self { id, fields, schema })
    }

    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    #[must_use]
    pub const fn fields(&self) -> &[String] {
        self.fields.as_slice()
    }

    #[must_use]
    pub fn at(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(String::as_str)
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.schema.index_of(name).and_then(|i| self.at(i))
    }

    #[must_use]
    pub const fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }
}
