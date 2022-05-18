
use std::{mem::size_of, slice::SliceIndex, ops::{RangeBounds, Bound}};
use wgpu::Buffer;
use crate::*;


#[derive(Debug)]
pub struct VertexBuffer<T: Clone>{
  data: Vec<T>,
  buffer: Buffer,
  max_size: usize,
}


fn write_into_vec<T: Clone>(target: &mut Vec<T>, offset: usize, source: &[T]) {

  let target_len = target.len();

  if offset > target_len {
    panic!("offset `{}` > traget.len() `{}`", offset, target_len);
  }

  let source_len = source.len();
  let write_len = (target_len - offset).min(source_len);

  if write_len > 0 {
    target[offset..offset+write_len].iter_mut().zip(&source[0..write_len]).for_each(|(entry, new_entry)| {
      *entry = new_entry.clone()
    });
  }

  if source_len - write_len > 0 {
    target.extend_from_slice(&source[write_len..]);
  }
}


impl<T: Clone> VertexBuffer<T> {

  pub fn new(gx: &Wgx, max_size: usize) -> Self {
    Self {
      data: Vec::with_capacity(max_size as usize),
      buffer: gx.buffer(BufUse::VERTEX | BufUse::COPY_DST, (size_of::<T>() * max_size) as u64, false),
      max_size,
    }
  }

  pub fn write_vertices(&mut self, offset: Option<usize>, vertices: &[T]) -> usize {
    if let Some(offset) = offset {
      write_into_vec(&mut self.data, offset, vertices);
    } else {
      self.data.extend_from_slice(vertices);
    }
    vertices.len()
  }

  pub fn clear(&mut self, retain: Option<usize>) {
    if let Some(size) = retain {
      self.data.truncate(size);
    } else {
      self.data.clear();
    }
  }

  pub fn write_buffer(&mut self, gx: &Wgx, range: impl SliceIndex<[T], Output = [T]> + RangeBounds<usize> + Clone) {

    let data_slice = &self.data[range.clone()];

    if !data_slice.is_empty() {

      let offset = match range.start_bound() {
        Bound::Included(start) => start * size_of::<T>(),
        Bound::Excluded(start) => (start + 1) * size_of::<T>(),
        Bound::Unbounded => 0,
      };

      gx.write_buffer(&self.buffer, offset as u64, data_slice);
    }
  }

  pub fn max_size(&self) -> usize { self.max_size }
  pub fn data(&self) -> &Vec<T> { &self.data }
  pub fn data_mut(&mut self) -> &mut Vec<T> { &mut self.data }
  pub fn len(&self) -> usize { self.data.len() }
  pub fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
  pub fn buffer_mut(&mut self) -> &mut wgpu::Buffer { &mut self.buffer }
}
