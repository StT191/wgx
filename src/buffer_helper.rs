
use std::{mem::size_of, ptr::copy_nonoverlapping, cmp::Ordering, ops::Range};
use wgpu::BufferAddress;
use crate::{*, error::*};


// range helper trait

pub trait TryToRange<O, E> {
  fn try_to(self) -> Result<Range<O>, E>;
}

impl<T, O: TryFrom<T>> TryToRange<O, <O as TryFrom<T>>::Error> for Range<T>
  where <O as TryFrom<T>>::Error: std::fmt::Debug
{
  fn try_to(self) -> Result<Range<O>, <O as TryFrom<T>>::Error> { Ok(Range {
    start: self.start.try_into()?,
    end: self.end.try_into()?,
  })}
}


// convert to byte ranges

pub const fn byte_range<T>(data_range: Range<usize>) -> Range<usize> {
  Range { start: data_range.start * size_of::<T>(), end: data_range.end * size_of::<T>() }
}

pub const fn buffer_range<T>(data_range: Range<usize>) -> Range<BufferAddress> {
  let byte_range = byte_range::<T>(data_range);
  Range { start: byte_range.start as BufferAddress, end: byte_range.end as BufferAddress }
}


// vec helper trait

pub trait CopyExtend<T: Copy> {
  fn copy_extend(&mut self, source: &[T], offset: Option<usize>) -> Range<usize>;
}

impl<T: Copy> CopyExtend<T> for Vec<T> {

  fn copy_extend(&mut self, source: &[T], offset: Option<usize>) -> Range<usize> {

    let start_len = self.len();
    let offset = offset.unwrap_or(start_len);

    match offset.cmp(&start_len) {
      Ordering::Less => {

        let end = offset + source.len();
        let need = isize::try_from(end).unwrap() - isize::try_from(start_len).unwrap();

        // SAFETY: We copy from a type that implements Copy.
        //         Pointers may never overlap because of rust aliasing rules for slices.
        //         We reserve enough space beforehand.
        //         We only set the new length of the vector after copying.
        unsafe {
          if need > 0 { self.reserve(need as usize); }

          copy_nonoverlapping(
            source.as_ptr(),
            self[offset..end].as_mut_ptr(),
            source.len(),
          );

          if need > 0 { self.set_len(end); }
        }

        offset..end
      },
      Ordering::Equal => {
        self.extend_from_slice(source);
        start_len..self.len()
      },
      Ordering::Greater => {
        panic!("offset `{offset}` > self.len() `{start_len}`")
      },
    }
  }
}


// wgpu draw indirect helper traits

pub trait DrawIndirectRanges: Sized {
  fn try_from_ranges(vertex_range: Range<usize>, instance_range: Range<usize>) -> Res<Self>;
  fn vertex_range(&self) -> Res<Range<u32>>;
  fn instance_range(&self) -> Res<Range<u32>>;
}

impl DrawIndirectRanges for DrawIndirectArgs {

  fn try_from_ranges(vertex_range: Range<usize>, instance_range: Range<usize>) -> Res<Self> {
    Ok(Self {
      first_vertex: u32::try_from(vertex_range.start).map_err(|_| "DrawIndirect first_vertex overflow")?,
      vertex_count: u32::try_from(vertex_range.len()).map_err(|_| "DrawIndirect vertex_count overflow")?,
      first_instance: u32::try_from(instance_range.start).map_err(|_| "DrawIndirect first_instance overflow")?,
      instance_count: u32::try_from(instance_range.len()).map_err(|_| "DrawIndirect instance_count overflow")?,
    })
  }

  fn vertex_range(&self) -> Res<Range<u32>> {
    Ok(self.first_vertex..self.first_vertex.checked_add(self.vertex_count).ok_or("DrawIndirect vertex range overflow")?)
  }

  fn instance_range(&self) -> Res<Range<u32>> {
    Ok(self.first_instance..self.first_instance.checked_add(self.instance_count).ok_or("DrawIndirect instance range overflow")?)
  }
}


pub trait DrawIndexedIndirectRanges: Sized {
  fn try_from_offset_ranges(vertex_offset: isize, index_range: Range<usize>, instance_range: Range<usize>) -> Res<Self>;
  fn index_range(&self) -> Res<Range<u32>>;
  fn instance_range(&self) -> Res<Range<u32>>;
}

impl DrawIndexedIndirectRanges for DrawIndexedIndirectArgs {

  fn try_from_offset_ranges(vertex_offset: isize, index_range: Range<usize>, instance_range: Range<usize>) -> Res<Self> {
    Ok(Self {
      base_vertex: i32::try_from(vertex_offset).map_err(|_| "DrawIndexedIndirect base_vertex overflow")?,
      first_index: u32::try_from(index_range.start).map_err(|_| "DrawIndexedIndirect first_index overflow")?,
      index_count: u32::try_from(index_range.len()).map_err(|_| "DrawIndexedIndirect index_count overflow")?,
      first_instance: u32::try_from(instance_range.start).map_err(|_| "DrawIndexedIndirect first_instance overflow")?,
      instance_count: u32::try_from(instance_range.len()).map_err(|_| "DrawIndexedIndirect instance_count overflow")?,
    })
  }

  fn index_range(&self) -> Res<Range<u32>> {
    Ok(self.first_index..self.first_index.checked_add(self.index_count).ok_or("DrawIndexedIndirect index range overflow")?)
  }

  fn instance_range(&self) -> Res<Range<u32>> {
    Ok(self.first_instance..self.first_instance.checked_add(self.instance_count).ok_or("DrawIndexedIndirect instance range overflow")?)
  }
}