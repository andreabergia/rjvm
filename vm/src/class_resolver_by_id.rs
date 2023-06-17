use crate::class::{ClassId, ClassRef};

pub trait ClassByIdResolver<'a> {
    fn find_class_by_id(&self, class_id: ClassId) -> Option<ClassRef<'a>>;
}
