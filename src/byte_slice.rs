
use core::{fmt, mem, slice, any::type_name};
use std::error::Error as StdError;


/// Possible errors during slice conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The input slice is not properly aligned for the
    /// output data type. E.g. for an `u32` output slice
    /// the memory must be 4-byte aligned.
    AlignmentMismatch {
        dst_type: &'static str,
        dst_minimum_alignment: usize,
    },
    /// A non-integer number of values from the output
    /// type would be in the output slice.
    LengthMismatch {
        dst_type: &'static str,
        src_slice_size: usize,
        dst_type_size: usize,
    },
    /*/// When converting a `Vec<T>` it had a capacity that
    /// allowed only for a non-integer number of values
    /// from the output type.
    CapacityMismatch {
        dst_type: &'static str,
        src_vec_capacity: usize,
        dst_type_capacity: usize,
    },*/
}

impl fmt::Display for Error {
    fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::AlignmentMismatch {
                dst_type,
                dst_minimum_alignment,
            } => {
                write!(
                    f,
                    "cannot cast a &[u8] into a &[{}]: the slice's address is not divisible by the minimum alignment ({}) of {}",
                    dst_type,
                    dst_minimum_alignment,
                    dst_type
                )?;
            }
            Error::LengthMismatch {
                dst_type,
                src_slice_size,
                dst_type_size,
            } => {
                write!(
                    f,
                    "cannot cast a &[u8] into a &[{}]: the size ({}) of the slice is not divisible by the size ({}) of {}",
                    dst_type,
                    src_slice_size,
                    dst_type_size,
                    dst_type
                )?;
            }
            /*Error::CapacityMismatch {
                dst_type,
                src_vec_capacity,
                dst_type_capacity,
            } => {
                write!(
                    f,
                    "cannot cast a vec into a vec::<{}>: the capacity ({}) of the vec is not divisible by the capacity of ({}) of {}",
                    dst_type,
                    src_vec_capacity,
                    dst_type_capacity,
                    dst_type
                )?;
            }*/
        }

        Ok(())
    }
}


impl StdError for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            AlignmentMismatch { .. } => "Alignment Mismatch",
            LengthMismatch { .. } => "Length Mismatch",
            // CapacityMismatch { .. } => "Capacity Mismatch",
        }
    }
}


fn check_alignment<T, U>(data: &T) -> Result<usize, Error>
where
    T: AsRef<[u8]> + ?Sized,
{
    let alignment = mem::align_of::<U>();

    if (data.as_ref().as_ptr() as usize) % alignment != 0 {
        let err = Error::AlignmentMismatch {
            dst_type: type_name::<U>(),
            dst_minimum_alignment: alignment,
        };
        return Err(err);
    }
    Ok(alignment)
}

fn check_length<T, U>(data: &T) -> Result<usize, Error>
where
    T: AsRef<[u8]> + ?Sized,
{
    let size_out = mem::size_of::<U>();
    if data.as_ref().len() % size_out != 0 {
        let err = Error::LengthMismatch {
            dst_type: type_name::<U>(),
            src_slice_size: data.as_ref().len(),
            dst_type_size: size_out,
        };
        return Err(err);
    }
    Ok(size_out)
}


fn check_constraints<U>(data: &[u8]) -> Result<usize, Error>
{
    if data.is_empty() {
        return Ok(0);
    }

    check_alignment::<[u8], U>(data)?;
    let size_out = check_length::<[u8], U>(data)?;

    Ok(data.len() / size_out)
}



pub unsafe trait FromByteSlice
where
    Self: Sized,
{
    fn from_byte_slice<T: AsRef<[u8]> + ?Sized>(slice: &T) -> Result<&[Self], Error>;
    fn from_mut_byte_slice<T: AsMut<[u8]> + ?Sized>(slice:&mut T) -> Result<&mut [Self], Error>;
}


pub unsafe trait ToByteSlice
where
    Self: Sized,
{
    fn to_byte_slice<T: AsRef<[Self]> + ?Sized>(slice: &T) -> &[u8];
}


pub unsafe trait ToMutByteSlice
where
    Self: Sized,
{
    fn to_mut_byte_slice<T: AsMut<[Self]> + ?Sized>(slice:&mut T) -> &mut [u8];
}


// implement

unsafe impl<U: Sized> FromByteSlice for U {

    fn from_byte_slice<T: AsRef<[u8]> + ?Sized>(slice: &T) -> Result<&[U], Error> {
        let slice = slice.as_ref();
        let len = check_constraints::<U>(slice)?;

        if len == 0 {
            Ok(&[])
        } else {
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                Ok(slice::from_raw_parts(slice.as_ptr() as *const U, len))
            }
        }
    }

    fn from_mut_byte_slice<T: AsMut<[u8]> + ?Sized>(slice:&mut T) -> Result<&mut [U], Error> {
        let slice = slice.as_mut();
        let len = check_constraints::<U>(slice)?;

        if len == 0 {
            Ok(&mut [])
        } else {
            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                Ok(slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut U, len))
            }
        }
    }
}

unsafe impl<U: Sized> ToByteSlice for U {

    fn to_byte_slice<T: AsRef<[U]> + ?Sized>(slice: &T) -> &[u8] {
        let slice = slice.as_ref();
        let len = slice.len() * mem::size_of::<U>();
        unsafe {
            slice::from_raw_parts(slice.as_ptr() as *const u8, len)
        }
    }
}

unsafe impl<U: Sized> ToMutByteSlice for U {

    fn to_mut_byte_slice<T: AsMut<[U]> + ?Sized>(slice:&mut T) -> &mut [u8] {
        let slice = slice.as_mut();
        let len = slice.len() * mem::size_of::<U>();
        unsafe {
            slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, len)
        }
    }
}





// pub implementations

pub trait AsSliceOf {
    fn as_slice_of<T: FromByteSlice>(&self) -> Result<&[T], Error>;
}

impl<U: AsRef<[u8]> + ?Sized> AsSliceOf for U {
    fn as_slice_of<T: FromByteSlice>(&self) -> Result<&[T], Error> {
        FromByteSlice::from_byte_slice(self)
    }
}


pub trait AsMutSliceOf {
    fn as_mut_slice_of<T: FromByteSlice>(&mut self) -> Result<&mut [T], Error>;
}

impl<U: AsMut<[u8]> + ?Sized> AsMutSliceOf for U {
    fn as_mut_slice_of<T: FromByteSlice>(&mut self) -> Result<&mut [T], Error> {
        FromByteSlice::from_mut_byte_slice(self)
    }
}


pub trait AsByteSlice<T> {
    fn as_byte_slice(&self) -> &[u8];
}

impl<T: ToByteSlice, U: AsRef<[T]> + ?Sized> AsByteSlice<T> for U {
    fn as_byte_slice(&self) -> &[u8] {
        ToByteSlice::to_byte_slice(self)
    }
}


pub trait AsMutByteSlice<T> {
    fn as_mut_byte_slice(&mut self) -> &mut [u8];
}

impl<T: ToMutByteSlice, U: AsMut<[T]> + ?Sized> AsMutByteSlice<T> for U {
    fn as_mut_byte_slice(&mut self) -> &mut [u8] {
        ToMutByteSlice::to_mut_byte_slice(self)
    }
}