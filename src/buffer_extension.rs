
use std::{sync::mpsc::sync_channel, ops::{RangeBounds}};
use wgpu::{Buffer, BufferSlice, BufferAddress};
use crate::{*, error::*};


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

      receiver.recv().convert()?.convert()?;

      cb(&buffer_slice);
    }

    self.unmap();

    Ok(())
  }
}