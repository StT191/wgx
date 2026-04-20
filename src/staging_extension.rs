
use std::{sync::mpsc::sync_channel, ops::RangeBounds};
use wgpu::{Buffer, BufferSlice, BufferAddress, BufferSize, BufferViewMut, WriteOnly, util::StagingBelt, CommandEncoder};
use crate::{*};
use anyhow::{Result as Res};


pub trait WriteOnlyExtension {
  fn write_data<T: ReadBytes>(&mut self, byte_offset: u64, data: T);
  fn write_data_iter<T: AsBytes, I: Iterator<Item=T>>(&mut self, byte_offset: u64, data: I) -> usize;
}

impl WriteOnlyExtension for WriteOnly<'_, [u8]> {

  fn write_data<T: ReadBytes>(&mut self, byte_offset: u64, data: T) {
    let bytes = data.read_bytes();
    if byte_offset == 0 && self.len() == bytes.len() {
      self.copy_from_slice(bytes);
    } else {
      let start = byte_offset as usize;
      self.slice(start..(start + bytes.len())).copy_from_slice(bytes);
    }
  }

  fn write_data_iter<T: AsBytes, I: Iterator<Item=T>>(&mut self, byte_offset: u64, data: I) -> usize {

    let mut ptr = self.as_raw_element_ptr().as_ptr();
    let mut count = 0;

    // SAFETY: Check that we don't write past dest!
    unsafe {
      let end = ptr.add(self.len());
      ptr = ptr.add(byte_offset as usize);

      for chunk in data {

        let stop = ptr.add(size_of::<T>());
        if stop > end { break; }

        ptr.copy_from_nonoverlapping(
          &chunk as *const T as *const u8,
          size_of::<T>(),
        );

        count += 1;
        ptr = stop;
      }
    }

    count
  }
}


pub trait StagingBeltExtension {

  fn stage(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, byte_range: impl RangeBounds<BufferAddress>,
  ) -> BufferViewMut;

  fn write_data<T: ReadBytes>(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, byte_offset: BufferAddress, data: T,
  );

  fn write_iter<T: AsBytes, I: Iterator<Item=T>>(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, byte_offset: BufferAddress, data: I,
  ) -> usize;
}

impl StagingBeltExtension for StagingBelt {

  fn stage(
    &mut self, encoder: &mut CommandEncoder,
    target: &Buffer, byte_range: impl RangeBounds<BufferAddress>,
  ) -> BufferViewMut {
    let byte_range = byte_range.map_into(0..target.size()).expect("byte-range can not exceed the target size");
    let byte_size = BufferSize::new(byte_range.end-byte_range.start).expect("write-buffer size can not be zero");
    self.write_buffer(encoder, target, byte_range.start, byte_size)
  }

  fn write_data<T: ReadBytes>(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, byte_offset: BufferAddress, data: T,
  ) {
    let bytes = data.read_bytes();
    self.stage(encoder, target, byte_offset..(bytes.len() as u64)).copy_from_slice(bytes);
  }

  fn write_iter<T: AsBytes, I: Iterator<Item=T>>(
    &mut self, encoder: &mut CommandEncoder,
    target: &Buffer, byte_offset: BufferAddress, data: I,
  ) -> usize {
    let hint = data.size_hint();
    let byte_size = hint.1.unwrap_or(hint.0) as u64 * size_of::<T>() as u64;
    let mut staging = self.stage(encoder, target, byte_offset..(byte_offset + byte_size));
    staging.slice(..).write_data_iter(0, data)
  }
}


#[derive(Debug)]
pub struct StagingEncoder {
  pub encoder: CommandEncoder,
  pub staging_belt: StagingBelt,
}

impl StagingEncoder {

  pub fn new(gx: &impl WgxDevice, chunk_size: u64) -> Self {
    Self {
      encoder: gx.command_encoder(),
      staging_belt: StagingBelt::new(gx.device().clone(), chunk_size),
    }
  }

  pub fn submit(&mut self, gx: &impl WgxDeviceQueue) {
    self.staging_belt.finish();
    gx.queue().submit([std::mem::replace(&mut self.encoder, gx.command_encoder()).finish()]);
    self.staging_belt.recall();
  }

  pub fn stage(&mut self, target: &Buffer, range: impl RangeBounds<BufferAddress>) -> BufferViewMut {
    self.staging_belt.stage(&mut self.encoder, target, range)
  }

  pub fn write_data<T: ReadBytes>(&mut self, target: &Buffer, offset: BufferAddress, data: T) {
    self.staging_belt.write_data(&mut self.encoder, target, offset, data)
  }

  pub fn write_iter<T: AsBytes, I>(&mut self, target: &Buffer, offset: BufferAddress, data: I) -> usize
    where I: Iterator<Item=T>
  {
    self.staging_belt.write_iter(&mut self.encoder, target, offset, data)
  }
}


pub trait WithMapSync {
  fn with_map_sync<T>(&self, gx: &impl WgxDevice, mode: MapMode, cb: impl FnOnce(&BufferSlice) -> T) -> Res<T>;
}

impl WithMapSync for BufferSlice<'_> {

  fn with_map_sync<T>(&self, gx: &impl WgxDevice, mode: MapMode, cb: impl FnOnce(&BufferSlice) -> T) -> Res<T> {
    let res = {
      let (sender, receiver) = sync_channel(1);

      self.map_async(mode, move |result| {
        let _ = sender.send(result);
      });

      gx.device().poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
      })?; // poll blocking

      receiver.recv()??;

      cb(self)
    };

    self.buffer().unmap();

    Ok(res)
  }
}