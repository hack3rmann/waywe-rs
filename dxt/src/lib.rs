use safe_transmute::{guard::SingleManyGuard, transmute_many, transmute_one_to_bytes};

pub enum DxtFormat {
    Dxt1,
    Dxt3,
    Dxt5,
}

pub fn decompress_image(width: usize, height: usize, data: &[u32], format: DxtFormat) -> Vec<u32> {
    let block_size = match format {
        DxtFormat::Dxt1 => 2,
        DxtFormat::Dxt3 | DxtFormat::Dxt5 => 4,
    };

    let num_blocks_width = (width + 3) / 4;
    let num_blocks_height = (height + 3) / 4;
    let expected_data_size = num_blocks_height * num_blocks_width * block_size;

    println!(
        "data_len: {}, expected_size:{expected_data_size}",
        data.len()
    );

    assert!(
        data.len() >= expected_data_size,
        "corrupt data in decompression, byte array length mismatched"
    );

    let mut res = vec![0; width * height];

    let mut block_idx = 0;
    for block_y in 0..num_blocks_height {
        for block_x in 0..num_blocks_width {
            let offset = block_idx * block_size;
            let data = &data[offset..offset + block_size];

            let x = block_x * 4;
            let y = block_y * 4;

            match format {
                DxtFormat::Dxt1 => decompress_dxt_1_block(data, &mut res, x, y, width, height),
                DxtFormat::Dxt3 => decompress_dxt_3_block(data, &mut res, x, y, width, height),
                DxtFormat::Dxt5 => decompress_dxt_5_block(data, &mut res, x, y, width, height),
            }

            block_idx += 1;
        }
    }

    res
}

#[rustfmt::skip]
fn get_color_palette(data: u32, format: DxtFormat) -> [u32; 4] {
    // TODO(ArnoDarkrose): handle error

    let colors = transmute_many::<u16, SingleManyGuard>(transmute_one_to_bytes(&data))
        .expect("failed to transmute u32 to u16 slice");

    let color0 = colors[0] as u32;
    let color1 = colors[1] as u32;

    // Convert RGB565 to RGB888
    let r0 = ((color0 >> 11) & 31) * 255 / 31;
    let g0 = ((color0 >> 5) & 63) * 255 / 63 ;
    let b0 = ((color0 & 31) * 255) / 31;

    let r1 = ((color1 >> 11) & 31) * 255 / 31;
    let g1 = ((color1 >> 5) & 63) * 255 / 63;
    let b1 = (color1 & 31) * 255 / 31;

    // NOTE(ArnoDarkrose): maybe make this from_be_bytes
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

fn decompress_dxt_1_block(
    data: &[u32],
    pixels: &mut [u32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let palette = get_color_palette(data[0], DxtFormat::Dxt1);

    let byte_view = transmute_one_to_bytes(&data[1]);
    for i in 0..4 {
        for j in 0..4 {
            if y + i < height && x + j < width {
                // Each pixel index (in block) takes 2 bits, so one byte can encode 4 indices
                let color_idx = (byte_view[i] >> (j * 2)) & 3;
                let color = palette[color_idx as usize];

                pixels[(y + i) * width + x + j] = color;
            }
        }
    }
}

fn decompress_dxt_3_block(
    data: &[u32],
    pixels: &mut [u32],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    let first_num_byte_view = transmute_one_to_bytes(&data[0]);
    let second_num_byte_view = transmute_one_to_bytes(&data[1]);

    let color_palette = get_color_palette(data[2], DxtFormat::Dxt3);

    let color_indices_byte_view = transmute_one_to_bytes(&data[3]);
    for i in 0..4 {
        for j in 0..4 {
            if y + i < height && x + j < width {
                // Each pixel index (in block) takes 2 bits, so one byte can encode 4 indices
                let color_idx = (color_indices_byte_view[i] >> (j * 2)) & 3;
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

                pixels[(y + i) * width + x + j] = color;
            }
        }
    }
}

fn decompress_dxt_5_block(
    data: &[u32],
    pixels: &mut [u32],
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
    for i in 0..4 {
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
                let color_idx = (color_indices_byte_view[i] >> (j * 2)) & 3;
                let mut color = color_palette[color_idx as usize];

                let alpha = alpha_palette[alpha_idx as usize];

                color &= u32::MAX >> 8;
                color |= (alpha as u32) << 24;

                pixels[(y + i) * width + x + j] = color;
            }

            cur_byte >>= 3;
            bits -= 3;
        }
    }
}

pub fn transmute_vec(mut src: Vec<u8>) -> Option<Vec<u32>> {
    if src.len() % 4 != 0 || src.capacity() % 4 != 0 {
        return None;
    }

    let ptr = src.as_ptr();

    if !(ptr.cast::<u32>().is_aligned()) {
        // TODO(ArnoDarkrose): handle error
        Some(
            src.chunks(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect(),
        )
    } else {
        // TODO(ArnoDarkrose): safety
        unsafe {
            let capacity = src.capacity();
            let len = src.len();
            let ptr = src.as_mut_ptr();
            std::mem::forget(src);
            Some(Vec::from_raw_parts(ptr.cast(), len / 4, capacity / 4))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

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
