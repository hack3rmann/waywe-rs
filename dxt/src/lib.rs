use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;
use safe_transmute::{guard::SingleManyGuard, transmute_many, transmute_one_to_bytes};

/// The format of the dxt image
pub enum DxtFormat {
    Dxt1,
    Dxt3,
    Dxt5,
}

/// Wrapper for *mut u32 to send it across threads
///
/// This is used to parllelize decompression and is only used
/// to get shared mutable access to non-overlapping
/// slices of the Vec
///
/// # Safety
///
/// - Send and Sync constraints are ensured by the implementation, so that
///   access to any vector element is exclusive at a time
/// - All threads that get this pointer finish their work after the main thread
///   that contains the vector itself
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct SendableU32MutPointer(*mut u32);
unsafe impl Send for SendableU32MutPointer {}
unsafe impl Sync for SendableU32MutPointer {}

/// Decompresses dxt image given its width, height and bytes
///
/// # Return value
///
/// Returns pixels of the decompressed picture in the format of `ABGR`
pub fn decompress_image(width: usize, height: usize, data: &[u32], format: DxtFormat) -> Vec<u32> {
    let block_size = match format {
        DxtFormat::Dxt1 => 2,
        DxtFormat::Dxt3 | DxtFormat::Dxt5 => 4,
    };

    // The picture is divided into 4x4 blocks
    let num_blocks_width = (width + 3) / 4;
    let num_blocks_height = (height + 3) / 4;
    let expected_data_size = num_blocks_height * num_blocks_width * block_size;

    assert!(
        data.len() >= expected_data_size,
        "corrupt data in decompression, byte array length mismatched"
    );

    let mut res = vec![0; width * height];
    let res_ptr = SendableU32MutPointer(res.as_mut_ptr());

    (0..num_blocks_height * num_blocks_width)
        .into_par_iter()
        .for_each(|block_idx| {
            let block_y = block_idx / num_blocks_width;
            let block_x = block_idx % num_blocks_width;

            let offset = block_idx * block_size;
            let data = &data[offset..offset + block_size];

            let x = block_x * 4;
            let y = block_y * 4;

            match format {
                DxtFormat::Dxt1 => decompress_dxt1_block(data, res_ptr, x, y, width, height),
                DxtFormat::Dxt3 => decompress_dxt3_block(data, res_ptr, x, y, width, height),
                DxtFormat::Dxt5 => decompress_dxt5_block(data, res_ptr, x, y, width, height),
            }
        });

    res
}

/// Extracts color palette encoded in u32
#[rustfmt::skip]
fn get_color_palette(data: u32, format: DxtFormat) -> [u32; 4] {
    let colors =
        transmute_many::<u16, SingleManyGuard>(transmute_one_to_bytes(&data))
            .expect("u32 can be safely transmuted to the slice of two u16");

    let color0 = colors[0] as u32;
    let color1 = colors[1] as u32;

    // Convert RGB565 to RGB888
    let r0 = ((color0 >> 11) & 31) * 255 / 31;
    let g0 = ((color0 >> 5) & 63) * 255 / 63 ;
    let b0 = ((color0 & 31) * 255) / 31;

    let r1 = ((color1 >> 11) & 31) * 255 / 31;
    let g1 = ((color1 >> 5) & 63) * 255 / 63;
    let b1 = (color1 & 31) * 255 / 31;

    let mut palette: [u32; 4] = [
        u32::from_le_bytes([r0 as u8, g0 as u8, b0 as u8, 255]),
        u32::from_le_bytes([r1 as u8, g1 as u8, b1 as u8, 255]),
        0,
        0,
    ];

    if color0 > color1 || matches!(format, DxtFormat::Dxt3 | DxtFormat::Dxt5) {
        palette[2] =
            u32::from_le_bytes([
                ((2 * r0 + r1) / 3) as u8,
                ((2 * g0 + g1) / 3) as u8,
                ((2 * b0 + b1) / 3) as u8,
                255
            ]);

        palette[3] =
            u32::from_le_bytes([
                ((r0 + 2 * r1) / 3) as u8,
                ((g0 + 2 * g1) / 3) as u8,
                ((b0 + 2 * b1) / 3) as u8,
                255
            ]);

    } else {
        palette[2] = u32::from_le_bytes([
            ((r0 + r1) / 2) as u8,
            ((g0 + g1) / 2) as u8,
            ((b0 + b1) / 2) as u8,
            255
        ]);
    }

    palette
}

/// Decompresses one block of the dxt1 image
///
/// # Format of the block:
///
/// - 2 bytes: color0 (in RGB565)
/// - 2 bytes: color1 (in RGB565)
/// - 4 bytes: color indices (2 bits per pixel)
fn decompress_dxt1_block(
    data: &[u32],
    out_pixels: SendableU32MutPointer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let palette = get_color_palette(data[0], DxtFormat::Dxt1);

    let byte_view = transmute_one_to_bytes(&data[1]);
    for (i, byte) in byte_view.iter().enumerate().take(4) {
        for j in 0..4 {
            if y + i < height && x + j < width {
                // Each pixel index (in block) takes 2 bits, so one byte can encode 4 indices
                let color_idx = (byte >> (j * 2)) & 3;
                let color = palette[color_idx as usize];

                // Safety
                //
                // Only one thread has access to this block at a time
                // so it is safe to dereference the pointer
                // and write to the pointee
                unsafe {
                    *out_pixels.0.add((y + i) * width + x + j) = color;
                }
            }
        }
    }
}

/// Decompresses one block of the dxt3 image
///
/// # Format of the block:
///
/// - 8 bytes: alpha values (4 bits per pixel)
/// - 8 bytes: color data (as in dxt1):
///   - 2 bytes: color0 (in RGB565)
///   - 2 bytes: color1 (in RGB565)
///   - 4 bytes: color indices (2 bits per pixel)
fn decompress_dxt3_block(
    data: &[u32],
    out_pixels: SendableU32MutPointer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let first_num_byte_view = transmute_one_to_bytes(&data[0]);
    let second_num_byte_view = transmute_one_to_bytes(&data[1]);

    let color_palette = get_color_palette(data[2], DxtFormat::Dxt3);

    let color_indices_byte_view = transmute_one_to_bytes(&data[3]);
    for (i, colors_byte) in color_indices_byte_view.iter().enumerate().take(4) {
        for j in 0..4 {
            if y + i < height && x + j < width {
                // Each pixel index (in block) takes 2 bits, so one byte can encode 4 indices
                let color_idx = (colors_byte >> (j * 2)) & 3;
                let mut color = color_palette[color_idx as usize];

                let idx = i * 4 + j;

                let byte = if idx < 4 {
                    first_num_byte_view[idx]
                } else {
                    second_num_byte_view[idx % 4]
                };

                let alpha = if idx % 2 == 0 {
                    (byte & 0xF) * 17
                } else {
                    (byte >> 4) * 17
                };

                color &= u32::MAX << 8;
                color |= alpha as u32;

                // Safety
                //
                // Only one thread has access to this block at a time
                // so it is safe to dereference the pointer
                // and write to the pointee
                unsafe {
                    *(out_pixels.0.add((y + i) * width + x + j)) = color;
                }
            }
        }
    }
}

/// Decompresses one block of the dxt5 image
///
/// # Format of the block:
///
/// - 1 byte: alpha0
/// - 1 byte: alpha1
/// - 6 bytes: alpha indices (3 bits per pixel)
/// - 8 bytes: color data (as in dxt1):
///   - 2 bytes: color0 (in RGB565)
///   - 2 bytes: color1 (in RGB565)
///   - 4 bytes: color indices (2 bits per pixel)
fn decompress_dxt5_block(
    data: &[u32],
    out_pixels: SendableU32MutPointer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let first_num_bytes_view = transmute_one_to_bytes(&data[0]);
    let second_num_bytes_view = transmute_one_to_bytes(&data[1]);

    let alpha0 = first_num_bytes_view[0] as u32;
    let alpha1 = first_num_bytes_view[1] as u32;

    let mut alpha_palette = [alpha0 as u8, alpha1 as u8, 0, 0, 0, 0, 0, 0];

    if alpha0 > alpha1 {
        for i in 0_u32..6 {
            alpha_palette[(i + 2) as usize] = (((6 - i) * alpha0 + (i + 1) * alpha1) / 7) as u8;
        }
    } else {
        for i in 0_u32..4 {
            alpha_palette[(i + 2) as usize] = (((4 - i) * alpha0 + (i + 1) * alpha1) / 5) as u8;
        }

        alpha_palette[7] = 255;
    }

    let color_palette = get_color_palette(data[2], DxtFormat::Dxt5);

    let mut bits = 0;
    let mut cur_byte = 0;
    let color_indices_byte_view = transmute_one_to_bytes(&data[3]);
    for (i, color_indices_byte) in color_indices_byte_view.iter().enumerate().take(4) {
        for j in 0..4 {
            if bits < 3 {
                let idx = 2 + (i * 4 + j) * 3 / 8;
                if idx < 4 {
                    cur_byte |= (first_num_bytes_view[idx] as u32) << bits;
                } else {
                    cur_byte |= (second_num_bytes_view[idx % 4] as u32) << bits;
                }
                bits += 8;
            }

            let alpha_idx = cur_byte & 7;

            if y + i < height && x + j < width {
                // Each pixel index (in block) takes 2 bits, so one byte can encode 4 indices
                let color_idx = (color_indices_byte >> (j * 2)) & 3;
                let mut color = color_palette[color_idx as usize];

                let alpha = alpha_palette[alpha_idx as usize];

                color &= u32::MAX >> 8;
                color |= (alpha as u32) << 24;

                // Safety
                //
                // Only one thread has access to this block at a time
                // so it is safe to dereference the pointer
                // and write to the pointee
                unsafe {
                    *out_pixels.0.add((y + i) * width + x + j) = color;
                }
            }

            cur_byte >>= 3;
            bits -= 3;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    use transmute_extra::transmute_vec;

    #[test]
    fn test_decopmpress_dxt5() {
        let mut gt_file = std::fs::File::open("mipmap.dat").unwrap();

        let mut gt = Vec::new();
        gt_file.read_to_end(&mut gt).unwrap();

        let gt = transmute_vec(gt).unwrap();

        let mut input_file = std::fs::File::open("decompress_input.dat").unwrap();
        let mut input = Vec::new();
        input_file.read_to_end(&mut input).unwrap();

        let mut input = transmute_vec(input).unwrap();

        let res = decompress_image(16, 24, &mut input, DxtFormat::Dxt5);

        for (i, (res, gt)) in res.into_iter().zip(gt.into_iter()).enumerate() {
            assert_eq!(res, gt, "i: {i}")
        }
    }
}
