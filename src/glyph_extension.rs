
use crate::{Wgx, DEPTH, RenderAttachment};
use cgmath::Matrix4;
use wgpu_glyph::{*, ab_glyph::{FontArc, InvalidFont}};
use wgpu::util::StagingBelt;



// extend Wgx
pub trait WgxGlyphBrushBuilderExtension {
    fn glyph_brush(&self, format:wgpu::TextureFormat, font_data:Vec<u8>)
        -> Result<GlyphBrush<(), FontArc>, InvalidFont>;

    fn glyph_brush_with_depth(&self, format:wgpu::TextureFormat, font_data:Vec<u8>)
        -> Result<GlyphBrush<wgpu::DepthStencilState, FontArc>, InvalidFont>;
}

impl WgxGlyphBrushBuilderExtension for Wgx {

    fn glyph_brush(&self, format:wgpu::TextureFormat, font_data:Vec<u8>)
        -> Result<GlyphBrush<(), FontArc>, InvalidFont>
    {
        let font = FontArc::try_from_vec(font_data)?;
        Ok(GlyphBrushBuilder::using_font(font).build(&self.device, format))
    }


    fn glyph_brush_with_depth(&self, format:wgpu::TextureFormat, font_data:Vec<u8>)
        -> Result<GlyphBrush<wgpu::DepthStencilState, FontArc>, InvalidFont>
    {
        let font = FontArc::try_from_vec(font_data)?;
        Ok(
            GlyphBrushBuilder::using_font(font)
            .depth_stencil_state(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .build(&self.device, format)
        )
    }
}


// layout macro
#[macro_export]
macro_rules! layout {
    (Single, $h:ident, $v:ident) => {
        wgpu_glyph::Layout::default_single_line()
        .h_align(wgpu_glyph::HorizontalAlign::$h)
        .v_align(wgpu_glyph::VerticalAlign::$v)
    };
    (Wrap, $h:ident, $v:ident) => {
        wgpu_glyph::Layout::default_wrap()
        .h_align(wgpu_glyph::HorizontalAlign::$h)
        .v_align(wgpu_glyph::VerticalAlign::$v)
    };
}


// glyphbrush extension
pub trait GlyphBrushExtension {
    fn load_font(&mut self, font_data:Vec<u8>) -> Result<FontId, InvalidFont>;
    fn add_text(
        &mut self, text:Vec<Text>, position:Option<(f32, f32)>, bounds:Option<(f32, f32)>,
        layout:Option<Layout<BuiltInLineBreaker>>
    );
}

impl<D> GlyphBrushExtension for GlyphBrush<D> {

    fn load_font(&mut self, font_data:Vec<u8>) -> Result<FontId, InvalidFont> {
        let font = FontArc::try_from_vec(font_data)?;
        Ok(self.add_font(font))
    }

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


// extend encoder
pub trait EncoderGlyphDrawExtension {
    fn draw_glyphs(
        &mut self, wgx:&Wgx, attachment:&RenderAttachment, glypths:&mut GlyphBrush<()>,
        transform:Matrix4<f32>, region:Option<[u32; 4]>, staging_belt:Option<&mut StagingBelt>,
    ) -> Result<(), String>;

    fn draw_glyphs_with_depth(
        &mut self, wgx:&Wgx, attachment:&RenderAttachment, glypths:&mut GlyphBrush<wgpu::DepthStencilState>,
        clear_depth:bool, transform:Matrix4<f32>, region:Option<[u32; 4]>, staging_belt:Option<&mut StagingBelt>,
    ) -> Result<(), String>;
}

impl EncoderGlyphDrawExtension for wgpu::CommandEncoder<> {

    fn draw_glyphs (
        &mut self, wgx:&Wgx, attachment:&RenderAttachment, glypths:&mut GlyphBrush<()>,
        transform:Matrix4<f32>, region:Option<[u32; 4]>, staging_belt:Option<&mut StagingBelt>,
    ) -> Result<(), String> {

        // staging_belt
        #![allow(unused_assignments)]
        let mut tmp_belt:Option<StagingBelt> = None;

        let staging_belt = if let Some(staging_belt) = staging_belt { staging_belt } else {
            tmp_belt = Some(StagingBelt::new(10240));
            tmp_belt.as_mut().unwrap()
        };


        let transform = transform * Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0);

        let result = if let Some(region) = region {
            glypths.draw_queued_with_transform_and_scissoring(
                &wgx.device, staging_belt, self, attachment.view, *transform.as_ref(),
                Region {x: region[0], y: region[1], width: region[2], height: region[3]}
            )
        }
        else {
            glypths.draw_queued_with_transform(
                &wgx.device, staging_belt, self, attachment.view, *transform.as_ref()
            )
        };

        result
    }

    fn draw_glyphs_with_depth(
        &mut self, wgx:&Wgx, attachment:&RenderAttachment, glypths:&mut GlyphBrush<wgpu::DepthStencilState>,
        clear_depth:bool, transform:Matrix4<f32>, region:Option<[u32; 4]>, staging_belt:Option<&mut StagingBelt>,
    ) -> Result<(), String> {

        #![allow(unused_assignments)]

        let depth_ops = wgpu::RenderPassDepthStencilAttachment {
            view: attachment.depth.ok_or("deph attachment missing")?,
            depth_ops: Some(wgpu::Operations {
                load: if clear_depth { wgpu::LoadOp::Clear(1.0) } else { wgpu::LoadOp::Load },
                store: true
            }),
            stencil_ops: None,
        };

        // staging_belt
        let mut tmp_belt:Option<StagingBelt> = None;

        let staging_belt = if let Some(staging_belt) = staging_belt { staging_belt } else {
            tmp_belt = Some(StagingBelt::new(10240));
            tmp_belt.as_mut().unwrap()
        };


        let transform = transform * Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0);

        let result = if let Some(region) = region {
            glypths.draw_queued_with_transform_and_scissoring(
                &wgx.device, staging_belt, self, attachment.view,
                depth_ops, *transform.as_ref(),
                Region {x: region[0], y: region[1], width: region[2], height: region[3]}
            )
        }
        else {
            glypths.draw_queued_with_transform(
                &wgx.device, staging_belt, self, attachment.view,
                depth_ops, *transform.as_ref()
            )
        };

        result
    }
}


