
#[macro_export]
macro_rules! binding {
    ($loc:expr, $stage:ident, UniformBuffer) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: wgpu::ShaderStage::$stage,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
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
        }
    };
    ($loc:expr, $stage:ident, Sampler) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: wgpu::ShaderStage::$stage,
            ty: wgpu::BindingType::Sampler { comparison: true },
        }
    };
}

#[macro_export]
macro_rules! bind {
    ($loc:expr, Buffer, $value:expr, $range:expr) => {
        wgpu::Binding {
            binding: $loc,
            resource: wgpu::BindingResource::Buffer {
                buffer: $value,
                range: $range,
            },
        }
    };
    ($loc:expr, $ty:ident, $value:expr) => {
        wgpu::Binding {
            binding: $loc,
            resource: wgpu::BindingResource::$ty($value),
        }
    };
}


#[macro_export]
macro_rules! vertex_desc {
    ($($loc:expr => $fmt:ident),*) => {
        wgpu::VertexBufferDescriptor {
            stride: ($(wgpu::vertex_format_size!($fmt) + )* 0) as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array!([] ; 0; $($loc => $fmt ,)*),
        }
    };
}