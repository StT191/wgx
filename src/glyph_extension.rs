
use cgmath::Matrix4;

use wgpu_glyph::{GlyphBrush, FontId, ab_glyph::{FontArc, InvalidFont}, Region, Section, Text};

use wgpu::util::StagingBelt;



// defs

pub trait GlyphExtension {
    fn add_text(&mut self, x:f32, y:f32, text:Vec<Text>);
}

pub trait GlyphLoadExtension {
    fn load_font(&mut self, font_data:Vec<u8>) -> Result<FontId, InvalidFont>;
}

pub trait GlyphDrawExtension {
    fn draw(
        &mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        attachment: &wgpu::TextureView, transform: Matrix4<f32>, region: Option<Region>
    ) -> Result<(), String>;
}

/*pub trait GlyphDrawWithDepthExtension {
    fn draw(
        &mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        attachment: &wgpu::TextureView, depth_attachment: &wgpu::TextureView, clear:bool,
        width: u32, height: u32
    ) -> Result<(), String>;
}*/



// impl

impl<D> GlyphExtension for GlyphBrush<D> {
    fn add_text(&mut self, x:f32, y:f32, text:Vec<Text>) {
        self.queue(Section {
            screen_position: (x, y),
            text,
            ..Section::default()
        });
    }
}


impl<D> GlyphLoadExtension for GlyphBrush<D> {
    fn load_font(&mut self, font_data:Vec<u8>) -> Result<FontId, InvalidFont> {
        let font = FontArc::try_from_vec(font_data)?;
        Ok(self.add_font(font))
    }
}



impl GlyphDrawExtension for GlyphBrush<()> {
    fn draw(
        &mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        attachment: &wgpu::TextureView, transform: Matrix4<f32>, _region: Option<Region>
    ) -> Result<(), String> {

        let mut belt = StagingBelt::new(4*1024*1024);



        /*let result = draw_queued_with_transform_and_scissoring(
            &mut self,
            device: &Device,
            staging_belt: &mut StagingBelt,
            encoder: &mut CommandEncoder,
            target: &TextureView,
            transform: [f32; 16],
            region: Region
        )*/
        /*let transform:[f32; 16] = [
            1.0/1000.0, 0.0, 0.0, 0.0,
            0.0, -1.0/1000.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];*/

        let result = self.draw_queued_with_transform(
            device, &mut belt, encoder, attachment, *transform.as_ref()
        );

        /*let result = self.draw_queued(
            device, &mut belt, encoder, attachment, 1000, 1000
        );*/

        // let _ = belt.recall();

        result
    }
}


/*wgpu::RenderPassDepthStencilAttachmentDescriptor {
    attachment: depth_attachment,
    depth_ops: Some(wgpu::Operations {
        load: if clear { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load },
        store: true
    }),
    stencil_ops: None,
},*/