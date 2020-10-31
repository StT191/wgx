
use cgmath::Matrix4;

use wgpu_glyph::{GlyphBrush, FontId, ab_glyph::{FontArc, InvalidFont}, Region, Section, Text, Layout, BuiltInLineBreaker};

use wgpu::util::StagingBelt;


// align macro

#[macro_export]
macro_rules! layout {
    (Single, $h:ident, $v:ident) => {
        wgpu_glyph::Layout::default_single_line()
        .h_align(HorizontalAlign::$h)
        .v_align(VerticalAlign::$v)
    };
    (Wrap, $h:ident, $v:ident) => {
        wgpu_glyph::Layout::default_wrap()
        .h_align(HorizontalAlign::$h)
        .v_align(VerticalAlign::$v)
    };
}


// defs

pub trait GlyphExtension {
    fn add_text(
        &mut self, text:Vec<Text>, position:Option<(f32, f32)>, bounds:Option<(f32, f32)>,
        layout:Option<Layout<BuiltInLineBreaker>>
    );
}

pub trait GlyphLoadExtension {
    fn load_font(&mut self, font_data:Vec<u8>) -> Result<FontId, InvalidFont>;
}

pub trait GlyphDrawExtension {
    fn draw(
        &mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        attachment: &wgpu::TextureView,
        transform: Matrix4<f32>, region: Option<[u32; 4]>
    ) -> Result<(), String>;
}

pub trait GlyphDrawWithDepthExtension {
    fn draw(
        &mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        view:(&wgpu::TextureView, &wgpu::TextureView), clear_depth:bool,
        transform: Matrix4<f32>, region: Option<[u32; 4]>
    ) -> Result<(), String>;
}



// impl

impl<D> GlyphExtension for GlyphBrush<D> {
    fn add_text(
        &mut self, text:Vec<Text>, position:Option<(f32, f32)>, bounds:Option<(f32, f32)>,
        layout:Option<Layout<BuiltInLineBreaker>>)
    {
        let mut section = Section { text, ..Section::default()};

        if let Some(position) = position { section = section.with_screen_position(position); }
        if let Some(bounds) = bounds { section = section.with_bounds(bounds); }
        if let Some(layout) = layout { section = section.with_layout(layout); }

        self.queue(section);
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
        attachment: &wgpu::TextureView,
        transform: Matrix4<f32>, region: Option<[u32; 4]>
    ) -> Result<(), String> {

        let mut belt = StagingBelt::new(4*1024*1024);

        let transform = transform * Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0);


        // draw

        let result = if let Some(region) = region {
            self.draw_queued_with_transform_and_scissoring(
                device, &mut belt, encoder, attachment, *transform.as_ref(),
                Region {x: region[0], y: region[1], width: region[2], height: region[3]}
            )
        }
        else {
            self.draw_queued_with_transform(
                device, &mut belt, encoder, attachment, *transform.as_ref()
            )
        };

        // let _ = belt.recall();

        result
    }
}


impl GlyphDrawWithDepthExtension for GlyphBrush<wgpu::DepthStencilStateDescriptor> {
    fn draw(
        &mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder,
        (attachment, depth_attachment):(&wgpu::TextureView, &wgpu::TextureView), clear_depth:bool,
        transform: Matrix4<f32>, region: Option<[u32; 4]>
    ) -> Result<(), String> {

        let mut belt = StagingBelt::new(4*1024*1024);

        let depth_ops = wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: depth_attachment,
            depth_ops: Some(wgpu::Operations {
                load: if clear_depth { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load },
                store: true
            }),
            stencil_ops: None,
        };

        let transform = transform * Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0);


        // draw

        let result = if let Some(region) = region {
            self.draw_queued_with_transform_and_scissoring(
                device, &mut belt, encoder, attachment,
                depth_ops, *transform.as_ref(),
                Region {x: region[0], y: region[1], width: region[2], height: region[3]}
            )
        }
        else {
            self.draw_queued_with_transform(
                device, &mut belt, encoder, attachment,
                depth_ops, *transform.as_ref()
            )
        };

        // let _ = belt.recall();

        result
    }
}