use rjvm_reader::field_type::{BaseType, FieldType};

use crate::{class::ClassId, class_resolver_by_id::ClassByIdResolver};

#[derive(PartialEq, Clone, Debug)]
#[repr(u8)]
// TODO: this should eventually be removed.
/// This models the entries of an array, and it is stored in the same memory as the entries.
/// Ideally, we'd want to reuse [FieldType], but unfortunately we cannot since it contains a
/// String, and its points to heap-allocated data. We could use modify that to use a raw
/// &str and create it from our memory chunk, but it would be complicated.
pub enum ArrayEntryType {
    Base(BaseType),
    Object(ClassId),
    // Note: here we would have to keep the sub-element type. Not doing this means that we do not
    // correctly support arrays of arrays!
    Array,
}

impl ArrayEntryType {
    pub fn into_field_type<'a>(
        self,
        class_resolver: &impl ClassByIdResolver<'a>,
    ) -> Option<FieldType> {
        match self {
            ArrayEntryType::Base(base_type) => Some(FieldType::Base(base_type)),
            ArrayEntryType::Object(class_id) => class_resolver
                .find_class_by_id(class_id)
                .map(|class| FieldType::Object(class.name.clone())),
            ArrayEntryType::Array => {
                todo!("Arrays of arrays are not supported at the moment")
            }
        }
    }
}
