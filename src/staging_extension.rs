
use std::{sync::mpsc::sync_channel, ops::{RangeBounds}};
use wgpu::{Buffer, BufferSlice, BufferAddress, BufferSize, BufferViewMut, WriteOnly, util::StagingBelt, CommandEncoder};
use crate::{*};
use anyhow::{Result as Res};


pub trait WriteOnlyExtension {
  fn write_data<T: ReadBytes>(&mut self, data: T);
  fn write_data_iter<T: ReadBytes, I: ExactSizeIterator<Item=T>>(&mut self, data: I);
}

impl WriteOnlyExtension for WriteOnly<'_, [u8]> {

  fn write_data<T: ReadBytes>(&mut self, data: T) {
    self.copy_from_slice(data.read_bytes())
  }

  fn write_data_iter<T: ReadBytes, I: ExactSizeIterator<Item=T>>(&mut self, data: I) {
    assert_eq!(data.len() * size_of::<T>(), self.len(), "the iterator needs to have the same len as the WriteOnly slice");
    let mut stop = 0;
    for (i, chunk) in data.enumerate() {
      let start = i * size_of::<T>();
      stop = start + size_of::<T>();
      assert!(stop <= self.len(), "the iterator exceeds the WriteOnly slice");
      self.slice(start..stop).write_data(chunk);
    }
    assert_eq!(stop, self.len(), "the iterator didn't fill the WriteOnly slice")
  }
}


pub trait StagingBeltExtension {

  fn stage(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, byte_range: impl RangeBounds<BufferAddress>,
  ) -> BufferViewMut;

  fn write_data<T: ReadBytes>(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, offset: BufferAddress, data: T,
  );

  fn write_iter<T: ReadBytes, I: ExactSizeIterator<Item=T>>(
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, offset: BufferAddress, data: I
  );
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
    &mut self, encoder: &mut CommandEncoder, target: &Buffer, offset: BufferAddress, data: T,
  ) {
    let bytes = data.read_bytes();
    self.stage(encoder, target, offset..(bytes.len() as u64)).copy_from_slice(bytes);
  }

  fn write_iter<T: ReadBytes, I: ExactSizeIterator<Item=T>>(
    &mut self, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, data: I
  ) {
    let size = (data.len() * size_of::<T>()) as u64;
    let mut staging = self.stage(encoder, target, offset..(offset + size));
    staging.slice(..).write_data_iter(data);
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

  pub fn write_iter<T: ReadBytes, I>(&mut self, target: &Buffer, offset: BufferAddress, data: I)
    where I: ExactSizeIterator<Item=T>
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