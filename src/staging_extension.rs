
use std::{sync::mpsc::sync_channel, ops::{RangeBounds}, iter::TrustedLen};
use wgpu::{Buffer, BufferSlice, BufferAddress, BufferSize, BufferViewMut, util::StagingBelt, CommandEncoder};
use crate::{*};
use anyhow::{Result as Res};


pub trait StagingBeltExtension {

  fn stage(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, byte_range: impl RangeBounds<BufferAddress>,
  ) -> BufferViewMut;

  fn write_data<T: ReadBytes>(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder, target: &Buffer, offset: BufferAddress, data: T,
  );

  fn write_iter<T: ReadBytes, I: TrustedLen<Item=T>>(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, data: I
  );
}

impl StagingBeltExtension for StagingBelt {

  fn stage(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, byte_range: impl RangeBounds<BufferAddress>,
  ) -> BufferViewMut {
    let byte_range = byte_range.map_into(0..target.size()).expect("byte-range can not exceed the target size");
    let byte_size = BufferSize::new(byte_range.end-byte_range.start).expect("write-buffer size can not be zero");
    self.write_buffer(encoder, target, byte_range.start, byte_size, gx.device())
  }

  fn write_data<T: ReadBytes>(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder, target: &Buffer, offset: BufferAddress, data: T,
  ) {
    let bytes = data.read_bytes();
    self.stage(gx, encoder, target, offset..(bytes.len() as u64)).copy_from_slice(bytes);
  }

  fn write_iter<T: ReadBytes, I: TrustedLen<Item=T>>(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, data: I
  ) {

    let size_hint = data.size_hint();
    assert_eq!(Some(size_hint.0), size_hint.1, "data doesn't provide a trusted size_hint: {size_hint:?}");
    let size = (size_hint.0 * size_of::<T>()) as u64;

    let mut staging = self.stage(gx, encoder, target, offset..(offset + size));
    T::write_iter(&mut staging, data);
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
      staging_belt: StagingBelt::new(chunk_size),
    }
  }

  pub fn submit(&mut self, gx: &impl WgxDeviceQueue) {
    self.staging_belt.finish();
    gx.queue().submit([std::mem::replace(&mut self.encoder, gx.command_encoder()).finish()]);
    self.staging_belt.recall();
  }

  pub fn stage(&mut self, gx: &impl WgxDevice, target: &Buffer, range: impl RangeBounds<BufferAddress>) -> BufferViewMut {
    self.staging_belt.stage(gx, &mut self.encoder, target, range)
  }

  pub fn write_data<T: ReadBytes>(&mut self, gx: &impl WgxDevice, target: &Buffer, offset: BufferAddress, data: T) {
    self.staging_belt.write_data(gx, &mut self.encoder, target, offset, data)
  }

  pub fn write_iter<T: ReadBytes, I>(&mut self, gx: &impl WgxDevice, target: &Buffer, offset: BufferAddress, data: I)
    where I: TrustedLen<Item=T>
  {
    self.staging_belt.write_iter(gx, &mut self.encoder, target, offset, data)
  }
}


pub trait WithMapSync {
  fn with_map_sync<S, T>(&self, gx: &impl WgxDevice, bounds: S, mode: MapMode, cb: impl FnOnce(&BufferSlice) -> T) -> Res<T>
    where S: RangeBounds<BufferAddress>;
}

impl WithMapSync for Buffer {

  fn with_map_sync<S, T>(&self, gx: &impl WgxDevice, bounds: S, mode: MapMode, cb: impl FnOnce(&BufferSlice) -> T) -> Res<T>
    where S: RangeBounds<BufferAddress>
  {
    let res = {
      let buffer_slice = self.slice(bounds);

      let (sender, receiver) = sync_channel(1);

      buffer_slice.map_async(mode, move |result| {
        let _ = sender.send(result);
      });

      gx.device().poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
      })?; // poll blocking

      receiver.recv()??;

      cb(&buffer_slice)
    };

    self.unmap();

    Ok(res)
  }
}