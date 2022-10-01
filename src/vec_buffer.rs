
use std::{mem::size_of, slice::SliceIndex, ops::{RangeBounds, Bound}, cmp::Ordering};
use wgpu::Buffer;
use crate::*;


#[derive(Debug)]
pub struct VecBuffer<T: Clone>{
  data: Vec<T>,
  buffer: Buffer,
  size: usize,
}


fn write_into_vec<T: Clone>(target: &mut Vec<T>, offset: usize, source: &[T]) {

  let target_len = target.len();

  match offset.cmp(&target_len) {
    Ordering::Less => {
      let source_len = source.len();
      let overwrite_len = (target_len - offset).min(source_len);

      target[offset..offset+overwrite_len].clone_from_slice(&source[..overwrite_len]);
      target.extend_from_slice(&source[overwrite_len..]);
    },
    Ordering::Equal => {
      target.extend_from_slice(&source);
    },
    Ordering::Greater => {
      panic!("offset `{offset}` > traget.len() `{target_len}`")
    },
  }
}


impl<T: Clone> VecBuffer<T> {

  pub fn new(gx: &Wgx, usage: wgpu::BufferUsages, size: usize) -> Self {
    Self {
      data: Vec::with_capacity(size as usize),
      buffer: gx.buffer(BufUse::COPY_DST | usage, (size_of::<T>() * size) as u64, false),
      size,
    }
  }

  pub fn write_vertex(&mut self, index: Option<usize>, vertex: &T) {
    if let Some(index) = index {
      let len = self.len();
      match index.cmp(&len) {
        Ordering::Less => self.data[index] = vertex.clone(),
        Ordering::Equal => self.data.push(vertex.clone()),
        Ordering::Greater => panic!("index `{index}` > traget.len() `{len}`"),
      }
    } else {
      self.data.push(vertex.clone());
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

  pub fn size(&self) -> usize { self.size }
  pub fn data(&self) -> &Vec<T> { &self.data }
  pub fn data_mut(&mut self) -> &mut Vec<T> { &mut self.data }
  pub fn len(&self) -> usize { self.data.len() }
  pub fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
  pub fn buffer_mut(&mut self) -> &mut wgpu::Buffer { &mut self.buffer }
}
