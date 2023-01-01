pub trait VecExt<T> {
    unsafe fn as_u8_slice(&self) -> &[u8];
}

impl<T> VecExt<T> for Vec<T>
where
    T: Sized,
{
    unsafe fn as_u8_slice(&self) -> &[u8] {
        let num_bytes = self.len() * std::mem::size_of::<T>();
        std::slice::from_raw_parts(self.as_ptr() as *const u8, num_bytes)
    }
}
