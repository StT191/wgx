
use std::{mem::size_of, slice::SliceIndex, ops::{Range, RangeBounds, Bound}, cmp::Ordering};
use crate::*;
pub use wgpu::{util::DrawIndirect};


#[derive(Debug)]
pub struct VecBuffer<T: Copy>{
  pub data: Vec<T>,
  pub buffer: wgpu::Buffer,
  size: usize,
}


fn write_into_vec<T: Copy>(target: &mut Vec<T>, offset: usize, source: &[T]) -> Range<usize> {

  let target_len = target.len();

  match offset.cmp(&target_len) {
    Ordering::Less => {

      let end = offset + source.len();
      let need = end as isize - target_len as isize;

      unsafe {
        if need > 0 {
          target.reserve(need as usize);
          target.set_len(end);
        }

        target[offset..end].copy_from_slice(&source);
      }

      offset..end
    },
    Ordering::Equal => {
      target.extend_from_slice(&source);
      target_len..target.len()
    },
    Ordering::Greater => {
      panic!("offset `{offset}` > traget.len() `{target_len}`")
    },
  }
}


impl<T: Copy> VecBuffer<T> {

  pub fn new(gx: &Wgx, usage: BufUse, size: usize) -> Self {
    Self {
      data: Vec::with_capacity(size),
      buffer: gx.buffer(BufUse::COPY_DST | usage, (size_of::<T>() * size) as u64, false),
      size,
    }
  }

  pub fn write(&mut self, index: Option<usize>, entry: &T) -> usize {
    if let Some(index) = index {
      let len = self.len();
      match index.cmp(&len) {
        Ordering::Less => self.data[index] = *entry,
        Ordering::Equal => self.data.push(*entry),
        Ordering::Greater => panic!("index `{index}` > traget.len() `{len}`"),
      }
      index
    } else {
      self.data.push(*entry);
      self.data.len()
    }
  }

  pub fn write_multiple(&mut self, offset: Option<usize>, entries: &[T]) -> Range<usize> {
    if let Some(offset) = offset {
      write_into_vec(&mut self.data, offset, entries)
    } else {
      let start = self.data.len();
      self.data.extend_from_slice(entries);
      start..self.data.len()
    }
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
  pub fn len(&self) -> usize { self.data.len() }
}




pub struct MultiDrawIndirect<Vertex:Copy, InstanceData:Copy> {
  pub vertices: VecBuffer<Vertex>,
  pub instances: VecBuffer<InstanceData>,
  pub indirect: VecBuffer<DrawIndirect>,
}

type Desc = (Option<BufUse>, usize); // (Buffer usages, max size)

impl<Vertex:Copy, InstanceData:Copy> MultiDrawIndirect<Vertex, InstanceData> {

  pub fn new(gx: &Wgx, vertex_desc: Desc, instance_desc: Desc, indirect_desc: Desc) -> Self {
    Self {
      vertices: VecBuffer::new(gx, vertex_desc.0.unwrap_or(BufUse::from_bits_truncate(0)) | BufUse::VERTEX, vertex_desc.1),
      instances: VecBuffer::new(gx, instance_desc.0.unwrap_or(BufUse::from_bits_truncate(0)) | BufUse::VERTEX, instance_desc.1),
      indirect: VecBuffer::new(gx, indirect_desc.0.unwrap_or(BufUse::from_bits_truncate(0)) | BufUse::INDIRECT, indirect_desc.1),
    }
  }

  pub fn write_buffers(&mut self, gx: &Wgx,
    vertices_range: impl SliceIndex<[Vertex], Output = [Vertex]> + RangeBounds<usize> + Clone,
    instances_range: impl SliceIndex<[InstanceData], Output = [InstanceData]> + RangeBounds<usize> + Clone,
    indirect_range: impl SliceIndex<[DrawIndirect], Output = [DrawIndirect]> + RangeBounds<usize> + Clone,
  ) {
    self.vertices.write_buffer(gx, vertices_range);
    self.instances.write_buffer(gx, instances_range);
    self.indirect.write_buffer(gx, indirect_range);
  }
}


pub trait DrawIndirectFromRanges {
  fn from_ranges(vertex_range: Range<usize>, instance_range: Range<usize>) -> Self;
}

impl DrawIndirectFromRanges for DrawIndirect {
  fn from_ranges(vertex_range: Range<usize>, instance_range: Range<usize>) -> Self {
    Self {
      base_vertex: vertex_range.start as u32,
      vertex_count: vertex_range.len() as u32,
      base_instance: instance_range.start as u32,
      instance_count: instance_range.len() as u32,
    }
  }
}