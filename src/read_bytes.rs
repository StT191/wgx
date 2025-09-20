
/// # Safety
/// must be guaranteed by implementor
pub unsafe trait ReadBytes {

    fn read_bytes(&self) -> &[u8] where Self: Sized {
        // SAFETY: must be guaranteed by implementor
        unsafe { core::slice::from_raw_parts(
            self as *const Self as *const u8,
            core::mem::size_of::<Self>(),
        ) }
    }

    fn write_iter(dest: &mut[u8], data: impl Iterator<Item=Self>) where Self: Sized {
      dest.chunks_mut(size_of::<Self>()).zip(data).for_each(|(c, d)| c.copy_from_slice(d.read_bytes()))
    }
}


// impls

unsafe impl<T: ReadBytes> ReadBytes for &T {
    fn read_bytes(&self) -> &[u8] { (*self).read_bytes() }
}


// slice types

unsafe impl<T: ReadBytes> ReadBytes for &[T] {
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by ReadBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            core::mem::size_of_val(*self),
        ) }
    }
}

unsafe impl<T: ReadBytes, const N: usize> ReadBytes for [T; N] {
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by ReadBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            N * core::mem::size_of::<T>(),
        ) }
    }
}


// plain types

macro_rules! impl_read_bytes {
    ($($type:ty),+ => $tokens:tt) => { $( unsafe impl ReadBytes for $type $tokens )+ }
}


impl_read_bytes!{
    (), crate::Color,
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
    => {}
}


use wgpu::util::{DrawIndirectArgs, DrawIndexedIndirectArgs, DispatchIndirectArgs};

impl_read_bytes!{
    DrawIndirectArgs, DrawIndexedIndirectArgs, DispatchIndirectArgs => {
        fn read_bytes(&self) -> &[u8] { self.as_bytes() }
    }
}


#[cfg(feature = "math")]
mod impl_read_bytes_for_math_types {
    use super::ReadBytes;

    use crate::math::{Vec3P, Mat3P};
    impl_read_bytes!{ Vec3P, Mat3P => {} }

    use glam::*;
    impl_read_bytes!{
        Mat2, Mat3, Mat4, Quat,
        Vec2, Vec3, Vec4,
        DAffine2, DAffine3,
        DMat2, DMat3, DMat4, DQuat,
        DVec2, DVec3, DVec4,
        I16Vec2, I16Vec3, I16Vec4,
        U16Vec2, U16Vec3, U16Vec4,
        IVec2, IVec3, IVec4,
        UVec2, UVec3, UVec4,
        I64Vec2, I64Vec3, I64Vec4,
        U64Vec2, U64Vec3, U64Vec4
        => {}
    }
}