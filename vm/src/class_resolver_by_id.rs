use crate::class::{ClassId, ClassRef};

/// Trait that models the fact that a class can be resolved by its given id
pub trait ClassByIdResolver<'a> {
    fn find_class_by_id(&self, class_id: ClassId) -> Option<ClassRef<'a>>;
}
