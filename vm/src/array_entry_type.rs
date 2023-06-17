use rjvm_reader::field_type::{BaseType, FieldType};

use crate::{class::ClassId, class_resolver_by_id::ClassByIdResolver};

#[derive(PartialEq, Clone, Debug)]
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
                // Arrays of arrays are not supported at the moment
                todo!()
            }
        }
    }
}
