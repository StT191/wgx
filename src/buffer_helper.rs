
use std::{mem::size_of, ptr::copy_nonoverlapping, cmp::Ordering};
use std::{fmt, ops::{Range, RangeBounds, Bound}, iter::Step};
use wgpu::BufferAddress;
use crate::{*};
use anyhow::{Result as Res, Context, bail};


// range helper

pub trait TryToRange<O, E> {
  fn try_to(self) -> Result<Range<O>, E>;
}

impl<T, O: TryFrom<T>> TryToRange<O, <O as TryFrom<T>>::Error> for Range<T>
  where <O as TryFrom<T>>::Error: fmt::Debug
{
  fn try_to(self) -> Result<Range<O>, <O as TryFrom<T>>::Error> { Ok(Range {
    start: self.start.try_into()?,
    end: self.end.try_into()?,
  })}
}


pub trait MapRange<T> {
  fn map_range<U>(self, map_fn: impl FnMut(T) -> U) -> Range<U>;
}

impl<T> MapRange<T> for Range<T> {
  fn map_range<U>(self, mut map_fn: impl FnMut(T) -> U) -> Range<U> {
    Range { start: map_fn(self.start), end: map_fn(self.end) }
  }
}


pub trait MapIntoRange<T> {
  fn map_into(&self, range: Range<T>) -> Res<Range<T>>;
}

impl<T: Step + fmt::Debug, R: RangeBounds<T>> MapIntoRange<T> for R {
  fn map_into(&self, range: Range<T>) -> Res<Range<T>> {
    use Bound::*;
    Ok(Range {
      start: match self.start_bound().cloned() {
        Unbounded => range.start.clone(),
        Included(start) => {
          if range.start <= start { start }
          else { bail!("start {:?} is out of range {:?}", self.start_bound(), &range) }
        },
        Excluded(border) => {
          let start = T::forward(border, 1);
          if range.start <= start { start }
          else { bail!("start {:?} is out of range {:?}", self.start_bound(), &range) }
        },
      },
      end: match self.end_bound().cloned() {
        Unbounded => range.end.clone(),
        Included(last) => {
          if last < range.end { T::forward(last, 1) }
          else { bail!("end {:?} is out of range {:?}", self.end_bound(), &range) }
        },
        Excluded(end) => {
          if end <= range.end { end }
          else { bail!("end {:?} is out of range {:?}", self.end_bound(), &range) }
        },
      },
    })
  }
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
      first_vertex: u32::try_from(vertex_range.start).context("DrawIndirect first_vertex overflow")?,
      vertex_count: u32::try_from(vertex_range.len()).context("DrawIndirect vertex_count overflow")?,
      first_instance: u32::try_from(instance_range.start).context("DrawIndirect first_instance overflow")?,
      instance_count: u32::try_from(instance_range.len()).context("DrawIndirect instance_count overflow")?,
    })
  }

  fn vertex_range(&self) -> Res<Range<u32>> {
    Ok(self.first_vertex..self.first_vertex.checked_add(self.vertex_count).context("DrawIndirect vertex range overflow")?)
  }

  fn instance_range(&self) -> Res<Range<u32>> {
    Ok(self.first_instance..self.first_instance.checked_add(self.instance_count).context("DrawIndirect instance range overflow")?)
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
      base_vertex: i32::try_from(vertex_offset).context("DrawIndexedIndirect base_vertex overflow")?,
      first_index: u32::try_from(index_range.start).context("DrawIndexedIndirect first_index overflow")?,
      index_count: u32::try_from(index_range.len()).context("DrawIndexedIndirect index_count overflow")?,
      first_instance: u32::try_from(instance_range.start).context("DrawIndexedIndirect first_instance overflow")?,
      instance_count: u32::try_from(instance_range.len()).context("DrawIndexedIndirect instance_count overflow")?,
    })
  }

  fn index_range(&self) -> Res<Range<u32>> {
    Ok(self.first_index..self.first_index.checked_add(self.index_count).context("DrawIndexedIndirect index range overflow")?)
  }

  fn instance_range(&self) -> Res<Range<u32>> {
    Ok(self.first_instance..self.first_instance.checked_add(self.instance_count).context("DrawIndexedIndirect instance range overflow")?)
  }
}