
use crate::*;
use std::num::NonZeroU32;


#[derive(Debug, Clone)]
pub struct FragmentStateConfig<'a, const N: usize> {
    module: &'a wgpu::ShaderModule,
    entry_point: &'a str,
    targets: [Option<wgpu::ColorTargetState>; N],
    compilation_options: wgpu::PipelineCompilationOptions<'a>,
}


#[derive(Debug, Clone)]
pub struct RenderPipelineConfig<'a, const N: usize> {

    pub label: wgpu::Label<'a>,

    pub layout: Option<wgpu::PipelineLayout>,

    pub vertex: wgpu::VertexState<'a>,
    pub primitive: wgpu::PrimitiveState,

    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: wgpu::MultisampleState,

    pub fragment: Option<FragmentStateConfig<'a, N>>,

    pub multiview: Option<NonZeroU32>,

    pub cache: Option<wgpu::PipelineCache>,
}

impl<'a> RenderPipelineConfig<'a, 0> {
    pub fn new(
        buffers: &'a [wgpu::VertexBufferLayout],
        module: &'a wgpu::ShaderModule, entry_point: &'a str,
        primitive: wgpu::PrimitiveState,
    ) -> Self {
        Self {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module,
                entry_point: if entry_point.is_empty() { None } else { Some(entry_point) },
                buffers,
                compilation_options: wgpu::PipelineCompilationOptions {
                    zero_initialize_workgroup_memory: false,
                    constants: Default::default(),
                },
            },
            primitive,
            fragment: None,
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
            multiview: None,
            cache: None,
        }
    }
}

impl<'a, const N: usize> RenderPipelineConfig<'a, N> {

    pub fn map<const C: usize>(self, map_fn: impl FnOnce(Self) -> RenderPipelineConfig<'a, C>) -> RenderPipelineConfig<'a, C> {
        map_fn(self)
    }

    pub fn conf(mut self, conf_fn: impl FnOnce(&mut Self)) -> Self { conf_fn(&mut self); self }

    pub fn label(mut self, label: wgpu::Label<'a>) -> Self { self.label = label; self }

    pub fn layout(mut self, layout: wgpu::PipelineLayout) -> Self { self.layout = Some(layout); self }

    pub fn pipeline_layout(self, gx: &impl WgxDevice, constants: &[wgpu::PushConstantRange], bind_groups: &[&wgpu::BindGroupLayout]) -> Self {
        self.layout(gx.pipeline_layout(constants, bind_groups))
    }

    pub fn msaa(mut self, msaa: u32) -> Self { self.multisample.count = msaa; self }

    pub fn vertex_shader_constants (self, constants: &'a ShaderConstants<'a>) -> Self {
        self.conf(|conf| conf.vertex.compilation_options.constants = constants)
    }

    pub fn fragment_shader_constants (mut self, constants: &'a ShaderConstants<'a>) -> Self {
        if let Some(fragment) = self.fragment.as_mut() {
            fragment.compilation_options.constants = constants;
        }
        self
    }

    pub fn depth_testing(mut self, format: TexFmt) -> Self {
        if let Some(depth) = self.depth_stencil.as_mut() {
            depth.format = format;
        } else {
            self.depth_stencil = Some(wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
        }
        self
    }

    pub fn depth_conf(mut self, conf_fn: impl FnOnce(&mut wgpu::DepthStencilState)) -> Self {
        if let Some(depth) = self.depth_stencil.as_mut() { conf_fn(depth) }
        self
    }

    pub fn fragment(mut self, module: &'a wgpu::ShaderModule, entry_point: &'a str) -> Self {
        self.fragment = Some(FragmentStateConfig {
            module,
            entry_point,
            targets: [const { None }; N],
            compilation_options: wgpu::PipelineCompilationOptions {
                zero_initialize_workgroup_memory: false,
                constants: Default::default(),
            },
        });
        self
    }

    pub fn fragment_conf(mut self, conf_fn: impl FnOnce(&mut FragmentStateConfig<N>)) -> Self {
        if let Some(fragment) = self.fragment.as_mut() { conf_fn(fragment) }
        self
    }

    pub fn target<const C: usize>(self, target: Option<wgpu::ColorTargetState>) -> RenderPipelineConfig<'a, C> {
        RenderPipelineConfig {
            label: self.label,
            cache: self.cache,
            layout: self.layout,
            vertex: self.vertex,
            primitive: self.primitive,
            depth_stencil: self.depth_stencil,
            multisample: self.multisample,
            multiview: self.multiview,
            fragment: self.fragment.map(|config| FragmentStateConfig {
                module: config.module,
                entry_point: config.entry_point,
                compilation_options: config.compilation_options,
                targets: {
                    let mut acc_targets = [const {None}; C];
                    for i in 0..N { acc_targets[i] = config.targets[i].clone() }
                    acc_targets[N] = target;
                    acc_targets
                },
            }),
        }
    }

    pub fn render_target<const C: usize>(self, render_target: &impl RenderTarget, blend: Option<Blend>, write_mask: wgpu::ColorWrites) -> RenderPipelineConfig<'a, C> {
        let config = self
            .target((render_target.format(), blend, write_mask).target())
            .msaa(render_target.msaa())
        ;
        if let Some(format) = render_target.depth_testing() {
            config.depth_testing(format)
        }
        else { config }
    }

    pub fn descriptor(&'a self) -> wgpu::RenderPipelineDescriptor<'a> {
        wgpu::RenderPipelineDescriptor {
            label: self.label,
            cache: self.cache.as_ref(),
            layout: self.layout.as_ref(),
            vertex: self.vertex.clone(),
            primitive: self.primitive,
            depth_stencil: self.depth_stencil.clone(),
            multisample: self.multisample,
            multiview: self.multiview,
            fragment: self.fragment.as_ref().map(|config| wgpu::FragmentState {
                module: config.module,
                entry_point: if config.entry_point.is_empty() { None } else { Some(config.entry_point) },
                targets: &config.targets,
                compilation_options: config.compilation_options.clone(),
            }),
        }
    }

    pub fn pipeline(&self, gx: &impl WgxDevice) -> wgpu::RenderPipeline {
        gx.render_pipeline(self)
    }
}



#[derive(Debug, Clone)]
pub struct ComputePipelineConfig<'a> {
    pub label: wgpu::Label<'a>,
    pub layout: Option<wgpu::PipelineLayout>,
    pub module: &'a wgpu::ShaderModule,
    pub entry_point: &'a str,
    pub compilation_options: wgpu::PipelineCompilationOptions<'a>,
    pub cache: Option<wgpu::PipelineCache>,
}

impl<'a> ComputePipelineConfig<'a> {

    pub fn new(module: &'a wgpu::ShaderModule, entry_point: &'a str) -> Self {
        Self {
            label: None, layout: None, cache: None,
            module, entry_point,
            compilation_options: wgpu::PipelineCompilationOptions {
                zero_initialize_workgroup_memory: false,
                constants: Default::default(),
            },
        }
    }

    pub fn conf(&mut self, map_fn: impl FnOnce(&mut Self)) -> &mut Self { map_fn(self); self }

    pub fn layout(&mut self, layout: wgpu::PipelineLayout) -> &mut Self { self.layout = Some(layout); self }

    pub fn pipeline_layout(&mut self, gx: &impl WgxDevice, constants: &[wgpu::PushConstantRange], bind_groups: &[&wgpu::BindGroupLayout]) -> &mut Self {
        self.layout(gx.pipeline_layout(constants, bind_groups))
    }

    pub fn shader_constants(&mut self, constants: &'a ShaderConstants<'a>) -> &mut Self {
        self.conf(|conf| conf.compilation_options.constants = constants)
    }

    pub fn descriptor(&'a self) -> wgpu::ComputePipelineDescriptor<'a> {
        wgpu::ComputePipelineDescriptor {
            label: self.label,
            layout: self.layout.as_ref(),
            module: self.module,
            entry_point: if self.entry_point.is_empty() { None } else { Some(self.entry_point) },
            compilation_options: self.compilation_options.clone(),
            cache: self.cache.as_ref(),
        }
    }

    pub fn pipeline(&self, gx: &impl WgxDevice) -> wgpu::ComputePipeline {
        gx.compute_pipeline(self)
    }
}