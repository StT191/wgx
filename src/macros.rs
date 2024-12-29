
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
                #[cfg(target_family = "wasm")] let limits = $crate::wgpu::Limits {
                    max_color_attachments: 4, // lower for some mobile browsers
                    ..$crate::wgpu::Limits::downlevel_webgl2_defaults()
                };
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
        $crate::binding!($loc, $stage,
            $crate::wgpu::BindingType::Buffer {
                ty: $crate::wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: ::core::num::NonZeroU64::new($min_size),
            },
            [$count]
        )
    };
    ($loc:expr, $stage:expr, StorageBuffer, $min_size:expr, $ro:expr) => {
        $crate::binding!($loc, $stage, StorageBuffer, $min_size, $ro, [0])
    };
    ($loc:expr, $stage:expr, StorageBuffer, $min_size:expr, $ro:expr, [$count:expr]) => {
        $crate::binding!($loc, $stage,
            $crate::wgpu::BindingType::Buffer {
                ty: $crate::wgpu::BufferBindingType::Storage { read_only: $ro },
                has_dynamic_offset: false,
                min_binding_size: ::core::num::NonZeroU64::new($min_size),
            },
            [$count]
        )
    };
    ($loc:expr, $stage:expr, Texture, $dim:ident, $sample_type:ident) => {
        $crate::binding!($loc, $stage, Texture, $dim, $sample_type, [0])
    };
    ($loc:expr, $stage:expr, Texture, $dim:ident, Float, [$count:expr]) => {
        $crate::binding!($loc, $stage, Texture, $dim, $crate::wgpu::TextureSampleType::Float { filterable: true }, false, [0])
    };
    ($loc:expr, $stage:expr, Texture, $dim:ident, $sample_type:ident, [$count:expr]) => {
        $crate::binding!($loc, $stage, Texture, $dim, $crate::wgpu::TextureSampleType::$sample_type, false, [0])
    };
    ($loc:expr, $stage:expr, MultisampledTexture, $dim:ident, $sample_type:ident) => {
        $crate::binding!($loc, $stage, MultisampledTexture, $dim, $sample_type, [0])
    };
    ($loc:expr, $stage:expr, MultisampledTexture, $dim:ident, Float, [$count:expr]) => {
        $crate::binding!($loc, $stage, Texture, $dim, $crate::wgpu::TextureSampleType::Float { filterable: true }, true, [0])
    };
    ($loc:expr, $stage:expr, MultisampledTexture, $dim:ident, $sample_type:ident, [$count:expr]) => {
        $crate::binding!($loc, $stage, Texture, $dim, $crate::wgpu::TextureSampleType::$sample_type, true, [0])
    };
    ($loc:expr, $stage:expr, Texture, $dim:ident, $sample_type:expr, $multisampled:expr, [$count:expr]) => {
        $crate::binding!($loc, $stage,
            $crate::wgpu::BindingType::Texture {
                view_dimension: $crate::wgpu::TextureViewDimension::$dim,
                sample_type: $sample_type,
                multisampled: $multisampled,
            },
            [$count]
        )
    };
    ($loc:expr, $stage:expr, StorageTexture, $dim:ident, $format:expr, $access:ident) => {
        $crate::binding!($loc, $stage, StorageTexture, $dim, $format, $access, [0])
    };
    ($loc:expr, $stage:expr, StorageTexture, $dim:ident, $format:expr, $access:ident, [$count:expr]) => {
        $crate::binding!($loc, $stage,
            $crate::wgpu::BindingType::StorageTexture {
                view_dimension: $crate::wgpu::TextureViewDimension::$dim,
                format: $format,
                access: $crate::wgpu::StorageTextureAccess::$access,
            },
            [$count]
        )
    };
    ($loc:expr, $stage:expr, Sampler, $binding_type:ident) => {
        $crate::binding!($loc, $stage, Sampler, $binding_type, [0])
    };
    ($loc:expr, $stage:expr, Sampler, $binding_type:ident, [$count:expr]) => {
        $crate::binding!($loc, $stage,
            $crate::wgpu::BindingType::Sampler($crate::wgpu::SamplerBindingType::$binding_type),
            [$count]
        )
    };
    ($loc:expr, $stage:expr, AccelerationStructure) => {
        $crate::binding!($loc, $stage, Sampler, $binding_type, [0])
    };
    ($loc:expr, $stage:expr, AccelerationStructure, [$count:expr]) => {
        $crate::binding!($loc, $stage,
            $crate::wgpu::BindingType::AccelerationStructure,
            [$count]
        )
    };
    ($loc:expr, $stage:expr, $ty:expr) => {
        $crate::binding!($loc, $stage, $ty, [0])
    };
    ($loc:expr, $stage:expr, $ty:expr, [$count:expr]) => {
        $crate::wgpu::BindGroupLayoutEntry {
            binding: $loc, visibility: $stage, ty: $ty,
            count: ::core::num::NonZeroU32::new($count),
        }
    };
}

#[macro_export]
macro_rules! bind_buffer {
    ($buffer:expr) => {
        $buffer.as_entire_buffer_binding()
    };
    ($buffer:expr, $offset:expr, $size:expr) => {
        $crate::wgpu::BufferBinding { buffer: $buffer, offset: $offset, size: $size }
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
            resource: $crate::wgpu::BindingResource::Buffer($crate::bind_buffer!($buffer, $offset, $size)),
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


#[macro_export]
macro_rules! shader_constants {
    ($($const:ident: $value:expr),*) => {{
        let capacity = 0 $( + {let _ = $value; 1} )*;
        let mut hash_map = ::std::collections::HashMap::<String, f64>::with_capacity(capacity);
        $(hash_map.insert(::std::stringify!($const).to_string(), $value as f64);)*
        hash_map
    }};
}
