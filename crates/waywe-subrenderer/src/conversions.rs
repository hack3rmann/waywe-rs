use abi_stable::std_types::{ROption, RStr};
use ash::vk;

pub(crate) fn map_vk_extent(extent: vk::Extent3D) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: extent.width,
        height: extent.height,
        depth_or_array_layers: extent.depth,
    }
}

pub(crate) fn map_wgpu_extent(extent: wgpu::Extent3d) -> vk::Extent3D {
    vk::Extent3D {
        width: extent.width,
        height: extent.height,
        depth: extent.depth_or_array_layers,
    }
}

pub(crate) fn map_ffi_label(label: ROption<RStr<'_>>) -> Option<&str> {
    match label {
        ROption::RSome(l) => Some(l.as_str()),
        ROption::RNone => None,
    }
}

pub(crate) fn map_wgpu_label(label: Option<&str>) -> ROption<RStr<'_>> {
    match label {
        Some(s) => ROption::RSome(RStr::from_str(s)),
        None => ROption::RNone,
    }
}

pub(crate) fn map_wgpu_dimension(dimension: wgpu::TextureDimension) -> vk::ImageType {
    match dimension {
        wgpu::TextureDimension::D1 => vk::ImageType::TYPE_1D,
        wgpu::TextureDimension::D2 => vk::ImageType::TYPE_2D,
        wgpu::TextureDimension::D3 => vk::ImageType::TYPE_3D,
    }
}

pub(crate) fn map_vk_dimension(ty: vk::ImageType) -> wgpu::TextureDimension {
    match ty {
        vk::ImageType::TYPE_1D => wgpu::TextureDimension::D1,
        vk::ImageType::TYPE_2D => wgpu::TextureDimension::D2,
        vk::ImageType::TYPE_3D => wgpu::TextureDimension::D3,
        _ => unreachable!("ImageType has only 3 actual variants"),
    }
}

pub(crate) fn map_vk_image_usage(usage: vk::ImageUsageFlags) -> wgpu::TextureUses {
    let mut bits = wgpu::TextureUses::empty();

    if usage.contains(vk::ImageUsageFlags::TRANSFER_SRC) {
        bits |= wgpu::TextureUses::COPY_SRC;
    }

    if usage.contains(vk::ImageUsageFlags::TRANSFER_DST) {
        bits |= wgpu::TextureUses::COPY_DST;
    }

    if usage.contains(vk::ImageUsageFlags::SAMPLED) {
        bits |= wgpu::TextureUses::RESOURCE;
    }

    if usage.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
        bits |= wgpu::TextureUses::COLOR_TARGET;
    }

    if usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
        bits |= wgpu::TextureUses::DEPTH_STENCIL_READ | wgpu::TextureUses::DEPTH_STENCIL_WRITE;
    }

    if usage.contains(vk::ImageUsageFlags::STORAGE) {
        bits |= wgpu::TextureUses::STORAGE_READ_ONLY
            | wgpu::TextureUses::STORAGE_WRITE_ONLY
            | wgpu::TextureUses::STORAGE_READ_WRITE
            | wgpu::TextureUses::STORAGE_ATOMIC;
    }

    bits
}

pub(crate) fn map_texture_usages_to_vk(
    usage: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
) -> vk::ImageUsageFlags {
    let is_depth_stencil = format.has_depth_aspect() || format.has_stencil_aspect();
    let mut flags = vk::ImageUsageFlags::empty();

    if usage.contains(wgpu::TextureUsages::COPY_SRC) {
        flags |= vk::ImageUsageFlags::TRANSFER_SRC;
    }

    if usage.contains(wgpu::TextureUsages::COPY_DST) {
        flags |= vk::ImageUsageFlags::TRANSFER_DST;
    }

    if usage.contains(wgpu::TextureUsages::TEXTURE_BINDING) {
        flags |= vk::ImageUsageFlags::SAMPLED;
    }

    if usage.contains(wgpu::TextureUsages::STORAGE_BINDING)
        || usage.contains(wgpu::TextureUsages::STORAGE_ATOMIC)
    {
        flags |= vk::ImageUsageFlags::STORAGE;
    }

    if usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
        if is_depth_stencil {
            flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        } else {
            flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
    }

    flags
}

pub(crate) fn map_vk_image_usage_to_texture_usages(
    usage: vk::ImageUsageFlags,
) -> wgpu::TextureUsages {
    let mut u = wgpu::TextureUsages::empty();

    if usage.contains(vk::ImageUsageFlags::TRANSFER_SRC) {
        u |= wgpu::TextureUsages::COPY_SRC;
    }

    if usage.contains(vk::ImageUsageFlags::TRANSFER_DST) {
        u |= wgpu::TextureUsages::COPY_DST;
    }

    if usage.contains(vk::ImageUsageFlags::SAMPLED) {
        u |= wgpu::TextureUsages::TEXTURE_BINDING;
    }

    if usage.contains(vk::ImageUsageFlags::STORAGE) {
        u |= wgpu::TextureUsages::STORAGE_BINDING;
    }

    if usage.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        || usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
    {
        u |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    u
}

pub(crate) fn _map_vk_format(format: vk::Format) -> Option<wgpu::TextureFormat> {
    use ash::vk::Format as F;
    use wgpu::TextureFormat as Tf;

    Some(match format {
        F::R8_UNORM => Tf::R8Unorm,
        F::R8_SNORM => Tf::R8Snorm,
        F::R8_UINT => Tf::R8Uint,
        F::R8_SINT => Tf::R8Sint,
        F::R16_UINT => Tf::R16Uint,
        F::R16_SINT => Tf::R16Sint,
        F::R16_UNORM => Tf::R16Unorm,
        F::R16_SNORM => Tf::R16Snorm,
        F::R16_SFLOAT => Tf::R16Float,
        F::R8G8_UNORM => Tf::Rg8Unorm,
        F::R8G8_SNORM => Tf::Rg8Snorm,
        F::R8G8_UINT => Tf::Rg8Uint,
        F::R8G8_SINT => Tf::Rg8Sint,
        F::R16G16_UNORM => Tf::Rg16Unorm,
        F::R16G16_SNORM => Tf::Rg16Snorm,
        F::R32_UINT => Tf::R32Uint,
        F::R32_SINT => Tf::R32Sint,
        F::R32_SFLOAT => Tf::R32Float,
        F::R16G16_UINT => Tf::Rg16Uint,
        F::R16G16_SINT => Tf::Rg16Sint,
        F::R16G16_SFLOAT => Tf::Rg16Float,
        F::R8G8B8A8_UNORM => Tf::Rgba8Unorm,
        F::R8G8B8A8_SRGB => Tf::Rgba8UnormSrgb,
        F::B8G8R8A8_SRGB => Tf::Bgra8UnormSrgb,
        F::R8G8B8A8_SNORM => Tf::Rgba8Snorm,
        F::B8G8R8A8_UNORM => Tf::Bgra8Unorm,
        F::R8G8B8A8_UINT => Tf::Rgba8Uint,
        F::R8G8B8A8_SINT => Tf::Rgba8Sint,
        F::A2B10G10R10_UINT_PACK32 => Tf::Rgb10a2Uint,
        F::A2B10G10R10_UNORM_PACK32 => Tf::Rgb10a2Unorm,
        F::B10G11R11_UFLOAT_PACK32 => Tf::Rg11b10Ufloat,
        F::R64_UINT => Tf::R64Uint,
        F::R32G32_UINT => Tf::Rg32Uint,
        F::R32G32_SINT => Tf::Rg32Sint,
        F::R32G32_SFLOAT => Tf::Rg32Float,
        F::R16G16B16A16_UINT => Tf::Rgba16Uint,
        F::R16G16B16A16_SINT => Tf::Rgba16Sint,
        F::R16G16B16A16_UNORM => Tf::Rgba16Unorm,
        F::R16G16B16A16_SNORM => Tf::Rgba16Snorm,
        F::R16G16B16A16_SFLOAT => Tf::Rgba16Float,
        F::R32G32B32A32_UINT => Tf::Rgba32Uint,
        F::R32G32B32A32_SINT => Tf::Rgba32Sint,
        F::R32G32B32A32_SFLOAT => Tf::Rgba32Float,
        F::D32_SFLOAT => Tf::Depth32Float,
        F::D32_SFLOAT_S8_UINT => Tf::Depth32FloatStencil8,
        F::X8_D24_UNORM_PACK32 => Tf::Depth24Plus,
        F::D24_UNORM_S8_UINT => Tf::Depth24PlusStencil8,
        F::S8_UINT => Tf::Stencil8,
        F::D16_UNORM => Tf::Depth16Unorm,
        F::G8_B8R8_2PLANE_420_UNORM => Tf::NV12,
        F::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 => Tf::P010,
        F::E5B9G9R9_UFLOAT_PACK32 => Tf::Rgb9e5Ufloat,
        F::BC1_RGBA_UNORM_BLOCK => Tf::Bc1RgbaUnorm,
        F::BC1_RGBA_SRGB_BLOCK => Tf::Bc1RgbaUnormSrgb,
        F::BC2_UNORM_BLOCK => Tf::Bc2RgbaUnorm,
        F::BC2_SRGB_BLOCK => Tf::Bc2RgbaUnormSrgb,
        F::BC3_UNORM_BLOCK => Tf::Bc3RgbaUnorm,
        F::BC3_SRGB_BLOCK => Tf::Bc3RgbaUnormSrgb,
        F::BC4_UNORM_BLOCK => Tf::Bc4RUnorm,
        F::BC4_SNORM_BLOCK => Tf::Bc4RSnorm,
        F::BC5_UNORM_BLOCK => Tf::Bc5RgUnorm,
        F::BC5_SNORM_BLOCK => Tf::Bc5RgSnorm,
        F::BC6H_UFLOAT_BLOCK => Tf::Bc6hRgbUfloat,
        F::BC6H_SFLOAT_BLOCK => Tf::Bc6hRgbFloat,
        F::BC7_UNORM_BLOCK => Tf::Bc7RgbaUnorm,
        F::BC7_SRGB_BLOCK => Tf::Bc7RgbaUnormSrgb,
        F::ETC2_R8G8B8_UNORM_BLOCK => Tf::Etc2Rgb8Unorm,
        F::ETC2_R8G8B8_SRGB_BLOCK => Tf::Etc2Rgb8UnormSrgb,
        F::ETC2_R8G8B8A1_UNORM_BLOCK => Tf::Etc2Rgb8A1Unorm,
        F::ETC2_R8G8B8A1_SRGB_BLOCK => Tf::Etc2Rgb8A1UnormSrgb,
        F::ETC2_R8G8B8A8_UNORM_BLOCK => Tf::Etc2Rgba8Unorm,
        F::ETC2_R8G8B8A8_SRGB_BLOCK => Tf::Etc2Rgba8UnormSrgb,
        F::EAC_R11_UNORM_BLOCK => Tf::EacR11Unorm,
        F::EAC_R11_SNORM_BLOCK => Tf::EacR11Snorm,
        F::EAC_R11G11_UNORM_BLOCK => Tf::EacRg11Unorm,
        F::EAC_R11G11_SNORM_BLOCK => Tf::EacRg11Snorm,
        // ASTC
        F::ASTC_4X4_UNORM_BLOCK => Tf::Astc {
            block: wgpu::AstcBlock::B4x4,
            channel: wgpu::AstcChannel::Unorm,
        },
        F::ASTC_4X4_SRGB_BLOCK => Tf::Astc {
            block: wgpu::AstcBlock::B4x4,
            channel: wgpu::AstcChannel::UnormSrgb,
        },
        F::ASTC_4X4_SFLOAT_BLOCK_EXT => Tf::Astc {
            block: wgpu::AstcBlock::B4x4,
            channel: wgpu::AstcChannel::Hdr,
        },
        // TODO(hack3rmann): repeat for other ASTC block sizes (B5x4, B5x5, B6x5, B6x6, B8x5, B8x6, B8x8, B10x5, B10x6, B10x8, B10x10, B12x10, B12x12)
        _ => return None,
    })
}

pub(crate) fn map_wgpu_format(format: wgpu::TextureFormat) -> Option<vk::Format> {
    use ash::vk::Format as F;
    use wgpu::wgt::{AstcBlock, AstcChannel, TextureFormat as Tf};

    Some(match format {
        Tf::R8Unorm => F::R8_UNORM,
        Tf::R8Snorm => F::R8_SNORM,
        Tf::R8Uint => F::R8_UINT,
        Tf::R8Sint => F::R8_SINT,
        Tf::R16Uint => F::R16_UINT,
        Tf::R16Sint => F::R16_SINT,
        Tf::R16Unorm => F::R16_UNORM,
        Tf::R16Snorm => F::R16_SNORM,
        Tf::R16Float => F::R16_SFLOAT,
        Tf::Rg8Unorm => F::R8G8_UNORM,
        Tf::Rg8Snorm => F::R8G8_SNORM,
        Tf::Rg8Uint => F::R8G8_UINT,
        Tf::Rg8Sint => F::R8G8_SINT,
        Tf::Rg16Unorm => F::R16G16_UNORM,
        Tf::Rg16Snorm => F::R16G16_SNORM,
        Tf::R32Uint => F::R32_UINT,
        Tf::R32Sint => F::R32_SINT,
        Tf::R32Float => F::R32_SFLOAT,
        Tf::Rg16Uint => F::R16G16_UINT,
        Tf::Rg16Sint => F::R16G16_SINT,
        Tf::Rg16Float => F::R16G16_SFLOAT,
        Tf::Rgba8Unorm => F::R8G8B8A8_UNORM,
        Tf::Rgba8UnormSrgb => F::R8G8B8A8_SRGB,
        Tf::Bgra8UnormSrgb => F::B8G8R8A8_SRGB,
        Tf::Rgba8Snorm => F::R8G8B8A8_SNORM,
        Tf::Bgra8Unorm => F::B8G8R8A8_UNORM,
        Tf::Rgba8Uint => F::R8G8B8A8_UINT,
        Tf::Rgba8Sint => F::R8G8B8A8_SINT,
        Tf::Rgb10a2Uint => F::A2B10G10R10_UINT_PACK32,
        Tf::Rgb10a2Unorm => F::A2B10G10R10_UNORM_PACK32,
        Tf::Rg11b10Ufloat => F::B10G11R11_UFLOAT_PACK32,
        Tf::R64Uint => F::R64_UINT,
        Tf::Rg32Uint => F::R32G32_UINT,
        Tf::Rg32Sint => F::R32G32_SINT,
        Tf::Rg32Float => F::R32G32_SFLOAT,
        Tf::Rgba16Uint => F::R16G16B16A16_UINT,
        Tf::Rgba16Sint => F::R16G16B16A16_SINT,
        Tf::Rgba16Unorm => F::R16G16B16A16_UNORM,
        Tf::Rgba16Snorm => F::R16G16B16A16_SNORM,
        Tf::Rgba16Float => F::R16G16B16A16_SFLOAT,
        Tf::Rgba32Uint => F::R32G32B32A32_UINT,
        Tf::Rgba32Sint => F::R32G32B32A32_SINT,
        Tf::Rgba32Float => F::R32G32B32A32_SFLOAT,
        Tf::Depth32Float => F::D32_SFLOAT,
        Tf::Depth32FloatStencil8 => F::D32_SFLOAT_S8_UINT,
        Tf::Depth24Plus => return None,
        Tf::Depth24PlusStencil8 => return None,
        Tf::Stencil8 => return None,
        Tf::Depth16Unorm => F::D16_UNORM,
        Tf::NV12 => F::G8_B8R8_2PLANE_420_UNORM,
        Tf::P010 => F::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16,
        Tf::Rgb9e5Ufloat => F::E5B9G9R9_UFLOAT_PACK32,
        Tf::Bc1RgbaUnorm => F::BC1_RGBA_UNORM_BLOCK,
        Tf::Bc1RgbaUnormSrgb => F::BC1_RGBA_SRGB_BLOCK,
        Tf::Bc2RgbaUnorm => F::BC2_UNORM_BLOCK,
        Tf::Bc2RgbaUnormSrgb => F::BC2_SRGB_BLOCK,
        Tf::Bc3RgbaUnorm => F::BC3_UNORM_BLOCK,
        Tf::Bc3RgbaUnormSrgb => F::BC3_SRGB_BLOCK,
        Tf::Bc4RUnorm => F::BC4_UNORM_BLOCK,
        Tf::Bc4RSnorm => F::BC4_SNORM_BLOCK,
        Tf::Bc5RgUnorm => F::BC5_UNORM_BLOCK,
        Tf::Bc5RgSnorm => F::BC5_SNORM_BLOCK,
        Tf::Bc6hRgbUfloat => F::BC6H_UFLOAT_BLOCK,
        Tf::Bc6hRgbFloat => F::BC6H_SFLOAT_BLOCK,
        Tf::Bc7RgbaUnorm => F::BC7_UNORM_BLOCK,
        Tf::Bc7RgbaUnormSrgb => F::BC7_SRGB_BLOCK,
        Tf::Etc2Rgb8Unorm => F::ETC2_R8G8B8_UNORM_BLOCK,
        Tf::Etc2Rgb8UnormSrgb => F::ETC2_R8G8B8_SRGB_BLOCK,
        Tf::Etc2Rgb8A1Unorm => F::ETC2_R8G8B8A1_UNORM_BLOCK,
        Tf::Etc2Rgb8A1UnormSrgb => F::ETC2_R8G8B8A1_SRGB_BLOCK,
        Tf::Etc2Rgba8Unorm => F::ETC2_R8G8B8A8_UNORM_BLOCK,
        Tf::Etc2Rgba8UnormSrgb => F::ETC2_R8G8B8A8_SRGB_BLOCK,
        Tf::EacR11Unorm => F::EAC_R11_UNORM_BLOCK,
        Tf::EacR11Snorm => F::EAC_R11_SNORM_BLOCK,
        Tf::EacRg11Unorm => F::EAC_R11G11_UNORM_BLOCK,
        Tf::EacRg11Snorm => F::EAC_R11G11_SNORM_BLOCK,
        Tf::Astc { block, channel } => match channel {
            AstcChannel::Unorm => match block {
                AstcBlock::B4x4 => F::ASTC_4X4_UNORM_BLOCK,
                AstcBlock::B5x4 => F::ASTC_5X4_UNORM_BLOCK,
                AstcBlock::B5x5 => F::ASTC_5X5_UNORM_BLOCK,
                AstcBlock::B6x5 => F::ASTC_6X5_UNORM_BLOCK,
                AstcBlock::B6x6 => F::ASTC_6X6_UNORM_BLOCK,
                AstcBlock::B8x5 => F::ASTC_8X5_UNORM_BLOCK,
                AstcBlock::B8x6 => F::ASTC_8X6_UNORM_BLOCK,
                AstcBlock::B8x8 => F::ASTC_8X8_UNORM_BLOCK,
                AstcBlock::B10x5 => F::ASTC_10X5_UNORM_BLOCK,
                AstcBlock::B10x6 => F::ASTC_10X6_UNORM_BLOCK,
                AstcBlock::B10x8 => F::ASTC_10X8_UNORM_BLOCK,
                AstcBlock::B10x10 => F::ASTC_10X10_UNORM_BLOCK,
                AstcBlock::B12x10 => F::ASTC_12X10_UNORM_BLOCK,
                AstcBlock::B12x12 => F::ASTC_12X12_UNORM_BLOCK,
            },
            AstcChannel::UnormSrgb => match block {
                AstcBlock::B4x4 => F::ASTC_4X4_SRGB_BLOCK,
                AstcBlock::B5x4 => F::ASTC_5X4_SRGB_BLOCK,
                AstcBlock::B5x5 => F::ASTC_5X5_SRGB_BLOCK,
                AstcBlock::B6x5 => F::ASTC_6X5_SRGB_BLOCK,
                AstcBlock::B6x6 => F::ASTC_6X6_SRGB_BLOCK,
                AstcBlock::B8x5 => F::ASTC_8X5_SRGB_BLOCK,
                AstcBlock::B8x6 => F::ASTC_8X6_SRGB_BLOCK,
                AstcBlock::B8x8 => F::ASTC_8X8_SRGB_BLOCK,
                AstcBlock::B10x5 => F::ASTC_10X5_SRGB_BLOCK,
                AstcBlock::B10x6 => F::ASTC_10X6_SRGB_BLOCK,
                AstcBlock::B10x8 => F::ASTC_10X8_SRGB_BLOCK,
                AstcBlock::B10x10 => F::ASTC_10X10_SRGB_BLOCK,
                AstcBlock::B12x10 => F::ASTC_12X10_SRGB_BLOCK,
                AstcBlock::B12x12 => F::ASTC_12X12_SRGB_BLOCK,
            },
            AstcChannel::Hdr => match block {
                AstcBlock::B4x4 => F::ASTC_4X4_SFLOAT_BLOCK_EXT,
                AstcBlock::B5x4 => F::ASTC_5X4_SFLOAT_BLOCK_EXT,
                AstcBlock::B5x5 => F::ASTC_5X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B6x5 => F::ASTC_6X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B6x6 => F::ASTC_6X6_SFLOAT_BLOCK_EXT,
                AstcBlock::B8x5 => F::ASTC_8X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B8x6 => F::ASTC_8X6_SFLOAT_BLOCK_EXT,
                AstcBlock::B8x8 => F::ASTC_8X8_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x5 => F::ASTC_10X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x6 => F::ASTC_10X6_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x8 => F::ASTC_10X8_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x10 => F::ASTC_10X10_SFLOAT_BLOCK_EXT,
                AstcBlock::B12x10 => F::ASTC_12X10_SFLOAT_BLOCK_EXT,
                AstcBlock::B12x12 => F::ASTC_12X12_SFLOAT_BLOCK_EXT,
            },
        },
    })
}
