
use std::ops::Range;
use bytemuck::Pod;

/// # Safety
/// Safety is guaranteed by the NoUninit binding.
pub unsafe trait AsBytes: Pod {

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        // SAFETY: is guranteed by the NoUninit binding.
        unsafe { core::slice::from_raw_parts(
            self as *const Self as *const u8,
            size_of::<Self>(),
        ) }
    }

    /// SAFETY: `dest` needs to be valid for writes of u8
    unsafe fn ptr_write_iter(dest: Range<*mut u8>, data: impl Iterator<Item=Self>) -> Result<usize, usize> {

        let Range { start: mut ptr, end } = dest;
        let mut count = 0;

        // SAFETY: Check that we don't write past end!
        unsafe {
            for chunk in data {

                let stop = ptr.add(size_of::<Self>());

                if stop > end {
                    return Err(count);
                }

                ptr.copy_from_nonoverlapping(
                    &chunk as *const Self as *const u8,
                    size_of::<Self>(),
                );

                count += 1;
                ptr = stop;
            }
        }

        Ok(count)
    }

    #[inline]
    fn write_iter(dest: &mut[u8], data: impl Iterator<Item=Self>) -> Result<usize, usize> {
        // SAFETY: `&mut[u8]` is always valid for writes of u8
        unsafe { Self::ptr_write_iter(dest.as_mut_ptr_range(), data) }
    }
}


unsafe impl<T: Pod> AsBytes for T {}


pub trait ReadBytes {

    fn read_bytes(&self) -> &[u8];

    #[inline]
    fn copy_bytes_to(&self, dest: &mut[u8]) {
      dest.copy_from_slice(self.read_bytes())
    }
}

impl<T: AsBytes> ReadBytes for &T {
    #[inline]
    fn read_bytes(&self) -> &[u8] { (*self).as_bytes() }
}

impl<T: AsBytes> ReadBytes for &[T] {
    #[inline]
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by AsBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            core::mem::size_of_val(*self),
        ) }
    }
}