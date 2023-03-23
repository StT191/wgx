
pub unsafe trait ReadBytes<V> {
    unsafe fn read_bytes(&self) -> &[u8];
}

// impl for any sliceable over a copy type

unsafe impl<T: AsRef<[V]>, V: Copy> ReadBytes<V> for T {
    unsafe fn read_bytes(&self) -> &[u8] {
        let slice = self.as_ref();
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<V>()
        )
    }
}