
#[macro_export]
macro_rules! binding {
    ($loc:expr, $stage:expr, UniformBuffer, $min_size:expr) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: core::num::NonZeroU64::new($min_size),
            },
            count: None,
        }
    };
    ($loc:expr, $stage:expr, StorageBuffer, $min_size:expr, $ro:expr) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: $ro },
                has_dynamic_offset: false,
                min_binding_size: core::num::NonZeroU64::new($min_size),
            },
            count: None,
        }
    };
    ($loc:expr, $stage:expr, SampledTexture) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    };
    ($loc:expr, $stage:expr, Sampler) => {
        wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: wgpu::BindingType::Sampler { comparison: false, filtering: true },
            count: None,
        }
    };
}


#[macro_export]
macro_rules! bind {
    ($loc:expr, Buffer, $buffer:expr) => {
        wgpu::BindGroupEntry {
            binding: $loc,
            resource: $buffer.as_entire_binding(),
        }
    };
    ($loc:expr, Buffer, $buffer:expr, $offset:expr, $size:expr) => {
        wgpu::BindGroupEntry {
            binding: $loc,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: $buffer,
                offset: $offset,
                size: $size,
            }),
        }
    };
    ($loc:expr, $ty:ident, $value:expr) => {
        wgpu::BindGroupEntry {
            binding: $loc,
            resource: wgpu::BindingResource::$ty($value),
        }
    };
}


#[macro_export]
macro_rules! vertex_desc {
    ($step:ident, $($loc:expr => $fmt:ident),*) => {
        wgpu::VertexBufferLayout {
            array_stride: ($(wgpu::VertexFormat::$fmt.size() + )* 0) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::$step,
            attributes: &wgpu::vertex_attr_array!([] ; 0; $($loc => $fmt ,)*),
        }
    };
}


#[macro_export]
macro_rules! push_constants {
    ($($range:expr => $stage:expr),*) => {
        &[$(wgpu::PushConstantRange {
            stages: $stage,
            range: $range,
        },)*]
    };
}

