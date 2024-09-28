
use std::{sync::mpsc::sync_channel, ops::{RangeBounds}};
use wgpu::{Buffer, BufferSlice, BufferAddress, BufferSize, BufferViewMut, util::StagingBelt, CommandEncoder};
use crate::{*};
use anyhow::{Result as Res};


pub trait StagingBeltExtension {
  fn map(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, size: u64,
  ) -> BufferViewMut;

  fn write_data<T: ReadBytes>(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, data: T,
  );
}

impl StagingBeltExtension for StagingBelt {

  fn map(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, size: u64,
  ) -> BufferViewMut {
    self.write_buffer(encoder, target, offset, BufferSize::new(size).unwrap(), gx.device())
  }

  fn write_data<T: ReadBytes>(
    &mut self, gx: &impl WgxDevice, encoder: &mut CommandEncoder,
    target: &Buffer, offset: BufferAddress, data: T,
  ) {
    let bytes = data.read_bytes();
    self.map(gx, encoder, target, offset, bytes.len() as u64).copy_from_slice(bytes);
  }
}


pub trait WithMapSync {
  fn with_map_sync<S>(&self, gx: &impl WgxDevice, bounds: S, mode: MapMode, cb: impl FnOnce(&BufferSlice)) -> Res<()>
    where S: RangeBounds<BufferAddress>;
}

impl WithMapSync for Buffer {

  fn with_map_sync<S>(&self, gx: &impl WgxDevice, bounds: S, mode: MapMode, cb: impl FnOnce(&BufferSlice)) -> Res<()>
    where S: RangeBounds<BufferAddress>
  {
    {
      let buffer_slice = self.slice(bounds);

      let (sender, receiver) = sync_channel(1);

      buffer_slice.map_async(mode, move |result| {
        let _ = sender.send(result);
      });

      gx.device().poll(wgpu::Maintain::Wait); // poll blocking

      receiver.recv()??;

      cb(&buffer_slice);
    }

    self.unmap();

    Ok(())
  }
}