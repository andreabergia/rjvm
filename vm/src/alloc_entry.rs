/// An allocation on our memory chunk
pub struct AllocEntry {
    pub(crate) ptr: *mut u8,
    pub(crate) alloc_size: usize,
}
