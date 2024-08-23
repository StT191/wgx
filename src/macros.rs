
#[macro_export]
macro_rules! features {
    ($($feature:ident),*) => {
        $crate::wgpu::Features::empty() $( | $crate::wgpu::Features::$feature )*
    };
}

#[macro_export]
macro_rules! limits {
    ($($key:ident: $value:expr),*) => {
        $crate::wgpu::Limits {
            $($key: $value, )*
            ..{
                #[cfg(not(target_family = "wasm"))] let limits = $crate::wgpu::Limits::default();
                #[cfg(target_family = "wasm")] let limits = $crate::wgpu::Limits::downlevel_webgl2_defaults();
                limits
            }
        }
    };
}


#[macro_export]
macro_rules! binding {
    ($loc:expr, $stage:expr, UniformBuffer, $min_size:expr) => {
        $crate::binding!($loc, $stage, UniformBuffer, $min_size, [0])
    };
    ($loc:expr, $stage:expr, UniformBuffer, $min_size:expr, [$count:expr]) => {
        $crate::wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: $crate::wgpu::BindingType::Buffer {
                ty: $crate::wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: ::core::num::NonZeroU64::new($min_size),
            },
            count: ::core::num::NonZeroU32::new($count),
        }
    };
    ($loc:expr, $stage:expr, StorageBuffer, $min_size:expr, $ro:expr) => {
        $crate::binding!($loc, $stage, StorageBuffer, $min_size, $ro, [0])
    };
    ($loc:expr, $stage:expr, StorageBuffer, $min_size:expr, $ro:expr, [$count:expr]) => {
        $crate::wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: $crate::wgpu::BindingType::Buffer {
                ty: $crate::wgpu::BufferBindingType::Storage { read_only: $ro },
                has_dynamic_offset: false,
                min_binding_size: ::core::num::NonZeroU64::new($min_size),
            },
            count: ::core::num::NonZeroU32::new($count),
        }
    };
    ($loc:expr, $stage:expr, Texture, $dim:ident) => {
        $crate::binding!($loc, $stage, Texture, $dim, [0])
    };
    ($loc:expr, $stage:expr, Texture, $dim:ident, [$count:expr]) => {
        $crate::wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: $crate::wgpu::BindingType::Texture {
                sample_type: $crate::wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: $crate::wgpu::TextureViewDimension::$dim,
                multisampled: false,
            },
            count: ::core::num::NonZeroU32::new($count),
        }
    };
    ($loc:expr, $stage:expr, Sampler) => {
        $crate::binding!($loc, $stage, Sampler, [0])
    };
    ($loc:expr, $stage:expr, Sampler, [$count:expr]) => {
        $crate::wgpu::BindGroupLayoutEntry {
            binding: $loc,
            visibility: $stage,
            ty: $crate::wgpu::BindingType::Sampler($crate::wgpu::SamplerBindingType::Filtering),
            count: ::core::num::NonZeroU32::new($count),
        }
    };
}


#[macro_export]
macro_rules! bind {
    ($loc:expr, Buffer, $buffer:expr) => {
        $crate::wgpu::BindGroupEntry {
            binding: $loc,
            resource: $buffer.as_entire_binding(),
        }
    };
    ($loc:expr, Buffer, $buffer:expr, $offset:expr, $size:expr) => {
        $crate::wgpu::BindGroupEntry {
            binding: $loc,
            resource: $crate::wgpu::BindingResource::Buffer($crate::wgpu::BufferBinding {
                buffer: $buffer,
                offset: $offset,
                size: $size,
            }),
        }
    };
    ($loc:expr, $ty:ident, $value:expr) => {
        $crate::wgpu::BindGroupEntry {
            binding: $loc,
            resource: $crate::wgpu::BindingResource::$ty($value),
        }
    };
}


#[macro_export]
macro_rules! vertex_dsc {
    ($step:ident, $($loc:expr => $fmt:ident),*) => {
        $crate::wgpu::VertexBufferLayout {
            array_stride: ($($crate::wgpu::VertexFormat::$fmt.size() + )* 0) as $crate::wgpu::BufferAddress,
            step_mode: $crate::wgpu::VertexStepMode::$step,
            attributes: &$crate::wgpu::vertex_attr_array!([] ; 0; $($loc => $fmt ,)*),
        }
    };
}


#[macro_export]
macro_rules! push_constants {
    ($($range:expr => $stage:expr),*) => {
        &[$($crate::wgpu::PushConstantRange {
            stages: $stage,
            range: $range,
        },)*]
    };
}

