
pub unsafe trait ReadBytes {
    fn read_bytes(&self) -> &[u8] where Self: Sized {
        // SAFETY: must be guaranteed by implementor
        unsafe { core::slice::from_raw_parts(
            self as *const Self as *const u8,
            core::mem::size_of::<Self>()
        ) }
    }
}


// impls

unsafe impl<T: ReadBytes> ReadBytes for &T {
    fn read_bytes(&self) -> &[u8] { (*self).read_bytes() }
}

unsafe impl<T: ReadBytes> ReadBytes for Option<T> {}


// slice types

unsafe impl<T: ReadBytes> ReadBytes for [T] {
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by ReadBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            self.len() * core::mem::size_of::<T>()
        ) }
    }
}

unsafe impl<T: ReadBytes> ReadBytes for &[T] {
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by ReadBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            self.len() * core::mem::size_of::<T>()
        ) }
    }
}

unsafe impl<T: ReadBytes, const N: usize> ReadBytes for [T; N] {
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by ReadBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            self.len() * core::mem::size_of::<T>()
        ) }
    }
}

unsafe impl<T: ReadBytes> ReadBytes for Vec<T> {
    fn read_bytes(&self) -> &[u8] {
        // SAFETY: guaranteed by ReadBytes binding
        unsafe { core::slice::from_raw_parts(
            self.as_ptr() as *const u8,
            self.len() * core::mem::size_of::<T>()
        ) }
    }
}


// plain types

macro_rules! impl_read_bytes {
    ($($type:ty),*) => {
        $( unsafe impl ReadBytes for $type {} )*
    }
}

use wgpu::util::{DrawIndirect, DrawIndexedIndirect};

impl_read_bytes!{
    (), crate::Color,
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64,
    DrawIndirect, DrawIndexedIndirect
}


#[cfg(feature = "projection")]
mod impl_proj {
    use super::ReadBytes;
    use cgmath::*;

    impl_read_bytes!{
        Matrix2<u8>, Matrix2<u16>, Matrix2<u32>, Matrix2<u64>, Matrix2<u128>, Matrix2<usize>, Matrix2<i8>, Matrix2<i16>, Matrix2<i32>, Matrix2<i64>, Matrix2<i128>, Matrix2<isize>, Matrix2<f32>, Matrix2<f64>,
        Matrix3<u8>, Matrix3<u16>, Matrix3<u32>, Matrix3<u64>, Matrix3<u128>, Matrix3<usize>, Matrix3<i8>, Matrix3<i16>, Matrix3<i32>, Matrix3<i64>, Matrix3<i128>, Matrix3<isize>, Matrix3<f32>, Matrix3<f64>,
        Matrix4<u8>, Matrix4<u16>, Matrix4<u32>, Matrix4<u64>, Matrix4<u128>, Matrix4<usize>, Matrix4<i8>, Matrix4<i16>, Matrix4<i32>, Matrix4<i64>, Matrix4<i128>, Matrix4<isize>, Matrix4<f32>, Matrix4<f64>,

        Vector1<u8>, Vector1<u16>, Vector1<u32>, Vector1<u64>, Vector1<u128>, Vector1<usize>, Vector1<i8>, Vector1<i16>, Vector1<i32>, Vector1<i64>, Vector1<i128>, Vector1<isize>, Vector1<f32>, Vector1<f64>,
        Vector2<u8>, Vector2<u16>, Vector2<u32>, Vector2<u64>, Vector2<u128>, Vector2<usize>, Vector2<i8>, Vector2<i16>, Vector2<i32>, Vector2<i64>, Vector2<i128>, Vector2<isize>, Vector2<f32>, Vector2<f64>,
        Vector3<u8>, Vector3<u16>, Vector3<u32>, Vector3<u64>, Vector3<u128>, Vector3<usize>, Vector3<i8>, Vector3<i16>, Vector3<i32>, Vector3<i64>, Vector3<i128>, Vector3<isize>, Vector3<f32>, Vector3<f64>,
        Vector4<u8>, Vector4<u16>, Vector4<u32>, Vector4<u64>, Vector4<u128>, Vector4<usize>, Vector4<i8>, Vector4<i16>, Vector4<i32>, Vector4<i64>, Vector4<i128>, Vector4<isize>, Vector4<f32>, Vector4<f64>,

        Rad<u8>, Rad<u16>, Rad<u32>, Rad<u64>, Rad<u128>, Rad<usize>, Rad<i8>, Rad<i16>, Rad<i32>, Rad<i64>, Rad<i128>, Rad<isize>, Rad<f32>, Rad<f64>,
        Deg<u8>, Deg<u16>, Deg<u32>, Deg<u64>, Deg<u128>, Deg<usize>, Deg<i8>, Deg<i16>, Deg<i32>, Deg<i64>, Deg<i128>, Deg<isize>, Deg<f32>, Deg<f64>,
        Euler<u8>, Euler<u16>, Euler<u32>, Euler<u64>, Euler<u128>, Euler<usize>, Euler<i8>, Euler<i16>, Euler<i32>, Euler<i64>, Euler<i128>, Euler<isize>, Euler<f32>, Euler<f64>,
        Quaternion<u8>, Quaternion<u16>, Quaternion<u32>, Quaternion<u64>, Quaternion<u128>, Quaternion<usize>, Quaternion<i8>, Quaternion<i16>, Quaternion<i32>, Quaternion<i64>, Quaternion<i128>, Quaternion<isize>, Quaternion<f32>, Quaternion<f64>,

        Point1<u8>, Point1<u16>, Point1<u32>, Point1<u64>, Point1<u128>, Point1<usize>, Point1<i8>, Point1<i16>, Point1<i32>, Point1<i64>, Point1<i128>, Point1<isize>, Point1<f32>, Point1<f64>,
        Point2<u8>, Point2<u16>, Point2<u32>, Point2<u64>, Point2<u128>, Point2<usize>, Point2<i8>, Point2<i16>, Point2<i32>, Point2<i64>, Point2<i128>, Point2<isize>, Point2<f32>, Point2<f64>,
        Point3<u8>, Point3<u16>, Point3<u32>, Point3<u64>, Point3<u128>, Point3<usize>, Point3<i8>, Point3<i16>, Point3<i32>, Point3<i64>, Point3<i128>, Point3<isize>, Point3<f32>, Point3<f64>
    }
}