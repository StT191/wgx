
use std::{mem::size_of, slice::SliceIndex, ops::{Range, RangeBounds, Bound}, cmp::Ordering};
use crate::{*, error::*};
pub use wgpu::{util::DrawIndirect};


#[derive(Debug)]
pub struct VecBuffer<T: Copy + ReadBytes>{
  pub data: Vec<T>,
  pub buffer: wgpu::Buffer,
  size: usize,
}


fn write_into_vec<T: Copy>(target: &mut Vec<T>, offset: usize, source: &[T]) -> Range<usize> {

  let target_len = target.len();

  match offset.cmp(&target_len) {
    Ordering::Less => {

      let end = offset + source.len();
      let need = isize::try_from(end).unwrap() - isize::try_from(target_len).unwrap() ;

      unsafe {
        if need > 0 {
          target.reserve(need as usize);

          // SAFETY: This is only safe because we'll copy over the uninitialized length right away
          //         and nothing needs to be dropped.
          target.set_len(end);
        }

        target[offset..end].copy_from_slice(source);
      }

      offset..end
    },
    Ordering::Equal => {
      target.extend_from_slice(source);
      target_len..target.len()
    },
    Ordering::Greater => {
      panic!("offset `{offset}` > traget.len() `{target_len}`")
    },
  }
}


impl<T: Copy + ReadBytes> VecBuffer<T> {

  pub fn new(gx: &impl WgxDevice, usage: BufUse, size: usize) -> Self {
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

  pub fn write_buffer(&self, gx: &impl WgxDeviceQueue, range: impl SliceIndex<[T], Output = [T]> + RangeBounds<usize> + Clone) {

    let data_slice = &self.data[range.clone()];

    if !data_slice.is_empty() {

      let offset = match range.start_bound() {
        Bound::Included(start) => start * size_of::<T>(),
        Bound::Excluded(start) => (start + 1) * size_of::<T>(),
        Bound::Unbounded => 0,
      };

      // may panic if getting larger than size
      gx.write_buffer(&self.buffer, offset as u64, data_slice);
    }
  }

  pub fn size(&self) -> usize { self.size }
  pub fn len(&self) -> usize { self.data.len() }
  pub fn is_empty(&self) -> bool { self.data.is_empty() }
}




pub struct MultiDrawIndirect<Vertex: Copy + ReadBytes, InstanceData: Copy + ReadBytes> {
  pub vertices: VecBuffer<Vertex>,
  pub instances: VecBuffer<InstanceData>,
  pub indirect: VecBuffer<DrawIndirect>,
}

type Desc = (Option<BufUse>, usize); // (Buffer usages, max size)

impl<Vertex: Copy + ReadBytes, InstanceData: Copy + ReadBytes> MultiDrawIndirect<Vertex, InstanceData> {

  pub fn new(gx: &impl WgxDevice, vertex_desc: Desc, instance_desc: Desc, indirect_desc: Desc) -> Self {
    Self {
      vertices: VecBuffer::new(gx, vertex_desc.0.unwrap_or(BufUse::empty()) | BufUse::VERTEX, vertex_desc.1),
      instances: VecBuffer::new(gx, instance_desc.0.unwrap_or(BufUse::empty()) | BufUse::VERTEX, instance_desc.1),
      indirect: VecBuffer::new(gx, indirect_desc.0.unwrap_or(BufUse::empty()) | BufUse::INDIRECT, indirect_desc.1),
    }
  }

  pub fn write_buffers(&mut self, gx: &impl WgxDeviceQueue,
    vertices_range: impl SliceIndex<[Vertex], Output = [Vertex]> + RangeBounds<usize> + Clone,
    instances_range: impl SliceIndex<[InstanceData], Output = [InstanceData]> + RangeBounds<usize> + Clone,
    indirect_range: impl SliceIndex<[DrawIndirect], Output = [DrawIndirect]> + RangeBounds<usize> + Clone,
  ) {
    self.vertices.write_buffer(gx, vertices_range);
    self.instances.write_buffer(gx, instances_range);
    self.indirect.write_buffer(gx, indirect_range);
  }
}


pub trait DrawIndirectFromRanges: Sized {
  fn from_ranges(vertex_range: Range<usize>, instance_range: Range<usize>) -> Res<Self>;
}

impl DrawIndirectFromRanges for DrawIndirect {
  fn from_ranges(vertex_range: Range<usize>, instance_range: Range<usize>) -> Res<Self> {
    Ok(Self {
      base_vertex: u32::try_from(vertex_range.start).map_err(|_| "DrawIndirect base_vertex overflow")?,
      vertex_count: u32::try_from(vertex_range.len()).map_err(|_| "DrawIndirect vertex_count overflow")?,
      base_instance: u32::try_from(instance_range.start).map_err(|_| "DrawIndirect base_instance overflow")?,
      instance_count: u32::try_from(instance_range.len()).map_err(|_| "DrawIndirect instance_count overflow")?,
    })
  }
}