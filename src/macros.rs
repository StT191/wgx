
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


/*#[macro_export]
macro_rules! vertex_attr_array {
    ($($loc:expr => $fmt:ident),*) => {
        $crate::vertex_attr_array!([] ; 0; $($loc => $fmt ,)*)
    };
    ([$($t:expr,)*] ; $off:expr ;) => { [$($t,)*] };
    ([$($t:expr,)*] ; $off:expr ; $loc:expr => $item:ident, $($ll:expr => $ii:ident ,)*) => {
        $crate::vertex_attr_array!(
            [$($t,)*
            wgpu::VertexAttributeDescriptor {
                format: wgpu::VertexFormat::$item,
                offset: $off,
                shader_location: $loc,
            },];
            $off + $crate::vertex_format_size!($item);
            $($ll => $ii ,)*
        )
    };
}


// For internal usage
#[macro_export]
macro_rules! vertex_format_size {
    (Uchar2) => { 2 };
    (Uchar4) => { 4 };
    (Char2) => { 2 };
    (Char4) => { 4 };
    (Uchar2Norm) => { 2 };
    (Uchar4Norm) => { 4 };
    (Char2Norm) => { 2 };
    (Char4Norm) => { 4 };
    (Ushort2) => { 4 };
    (Ushort4) => { 8 };
    (Short2) => { 4 };
    (Short4) => { 8 };
    (Ushort2Norm) => { 4 };
    (Ushort4Norm) => { 8 };
    (Short2Norm) => { 4 };
    (Short4Norm) => { 8 };
    (Half2) => { 4 };
    (Half4) => { 8 };
    (Float) => { 4 };
    (Float2) => { 8 };
    (Float3) => { 12 };
    (Float4) => { 16 };
    (Uint) => { 4 };
    (Uint2) => { 8 };
    (Uint3) => { 12 };
    (Uint4) => { 16 };
    (Int) => { 4 };
    (Int2) => { 8 };
    (Int3) => { 12 };
    (Int4) => { 16 };
}*/