
#[macro_export]
macro_rules! binding {
    ($loc:expr, $stage:ident, UniformBuffer, $min_size:expr) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: wgpu::ShaderStage::$stage,
            ty: wgpu::BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: core::num::NonZeroU64::new($min_size),
            },
            count: None,
        }
    };
    ($loc:expr, $stage:ident, SampledTexture) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: wgpu::ShaderStage::$stage,
            ty: wgpu::BindingType::SampledTexture {
                component_type: wgpu::TextureComponentType::Float,
                multisampled: false,
                dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }
    };
    ($loc:expr, $stage:ident, Sampler) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: wgpu::ShaderStage::$stage,
            ty: wgpu::BindingType::Sampler { comparison: false },
            count: None,
        }
    };
}

#[macro_export]
macro_rules! bind {
    ($loc:expr, $ty:ident, $value:expr) => {
        wgpu::BindGroupEntry {
            binding: $loc,
            resource: wgpu::BindingResource::$ty($value),
        }
    };
}


#[macro_export]
macro_rules! vertex_desc {
    ($($loc:expr => $fmt:ident),*) => {
        wgpu::VertexBufferDescriptor {
            stride: ($(wgpu::VertexFormat::$fmt.size() + )* 0) as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array!([] ; 0; $($loc => $fmt ,)*),
        }
    };
}

