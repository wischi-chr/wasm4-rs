use wasm4_common::draw::{BitsPerPixel, Sprite};

const NEW_LINE: u32 = 10;
const TAB: u32 = 9;
const SPACE: u32 = 32;

enum SpriteInfo<const N: usize> {
    Sprite(Sprite<[u8; N]>),
    Calc(BitsPerPixel, usize),
}

impl<const N: usize> SpriteInfo<N> {
    pub const fn calc(self) -> (BitsPerPixel, usize) {
        match self {
            SpriteInfo::Calc(bpp, capacity) => (bpp, capacity),
            SpriteInfo::Sprite(_) => unreachable!(),
        }
    }

    pub const fn sprite(self) -> Sprite<[u8; N]> {
        match self {
            SpriteInfo::Sprite(sprite) => sprite,
            SpriteInfo::Calc(_, _) => unreachable!(),
        }
    }
}

pub const fn sprite_calc(input: &str) -> (BitsPerPixel, usize) {
    sprite_builder_impl::<0>(BitsPerPixel::Two, true, input).calc()
}

pub const fn sprite_builder<const N: usize>(input: &str) -> Sprite<[u8; N]> {
    let (bpp, capacity) = sprite_calc(input);

    if capacity != N {
        panic!("Inconsistent capacity");
    }

    sprite_builder_impl::<N>(bpp, false, input).sprite()
}

const fn sprite_builder_impl<const N: usize>(
    bpp: BitsPerPixel,
    calc_capacity: bool,
    input: &str,
) -> SpriteInfo<N> {
    let input = input.as_bytes().split_at(1).1;

    let mut output_buffer: u8 = 0;
    let mut output_buffer_bits: u8 = 0;

    let input_len = input.len();
    let mut input_offset: usize = 0;

    let mut output: [u8; N] = [0; N];
    let mut output_offset: usize = 0;

    let mut pattern_chars = [0u32; 4];
    let mut pattern_chars_len: usize = 0;

    let mut input_first = false;
    let mut previous_char: u32 = 0;

    let mut width: Option<u32> = None;
    let mut current_width: u32 = 0;
    let mut current_height: u32 = 0;

    let bit_pixel_shift: u8 = match bpp {
        BitsPerPixel::One => 1,
        BitsPerPixel::Two => 2,
    };

    // now read str input
    loop {
        let sub_input = input.split_at(input_offset).1;
        let res = decode_utf8_first(sub_input);
        input_offset += res.0;

        if res.1 == SPACE || res.1 == TAB {
            // we ignore whitespace to allow for indentation

            if input_offset == input_len {
                break;
            }

            continue;
        }

        if res.1 == NEW_LINE {
            if width.is_none() {
                width = Some(current_width);
            }

            let expected_width = width.unwrap();

            if current_width != expected_width {
                panic!("Not all rows are the same width.");
            }

            current_height += 1;

            if input_offset == input_len {
                break;
            }

            current_width = 0;

            continue;
        }

        input_first = !input_first;

        if input_first {
            previous_char = res.1;
            continue;
        }

        if previous_char != res.1 {
            panic!("pattern pairs not matching");
        }

        // lookup in patterns to get index
        let mut text_idx = 0;

        let pattern_index: u8 = loop {
            if text_idx >= pattern_chars_len {
                // not found in pattern list, now we have to
                // determine if there is space for more or if we panic

                if text_idx >= 4 {
                    panic!("failed to find char in pattern");
                }

                pattern_chars[text_idx] = res.1;
                pattern_chars_len += 1;
            }

            if pattern_chars[text_idx] == res.1 {
                break text_idx as u8;
            }

            text_idx += 1;
        };

        current_width += 1;

        if output_buffer_bits == 8 {
            if output_offset < N {
                output[output_offset] = output_buffer;
            }

            output_offset += 1;

            output_buffer = 0;
            output_buffer_bits = 0;
        }

        output_buffer = output_buffer
            .checked_shl(bit_pixel_shift as u32)
            .expect("Expect shift to work");

        output_buffer |= pattern_index;
        output_buffer_bits += bit_pixel_shift;
    }

    if output_offset < N {
        output[output_offset] = output_buffer;
    }

    current_width = width.unwrap();

    if calc_capacity {
        let (bpp, div) = if pattern_chars_len <= 2 {
            (BitsPerPixel::One, 8)
        } else {
            (BitsPerPixel::Two, 4)
        };

        SpriteInfo::Calc(bpp, (current_width * current_height).div_ceil(div) as usize)
    } else {
        SpriteInfo::Sprite(
            Sprite::<[u8; N]>::from_byte_array(output, [current_width, current_height], bpp)
                .expect("Creating sprite should work"),
        )
    }
}

/// Decode the first UTF-8 code point from a byte slice into a u32
/// in a const-friendly way.
const fn decode_utf8_first(bytes: &[u8]) -> (usize, u32) {
    let first = bytes[0];

    // Single-byte (ASCII): 0xxxxxxx
    if first & 0b1000_0000 == 0 {
        return (1, first as u32);
    }

    // Two-byte sequence: 110xxxxx 10xxxxxx
    if first & 0b1110_0000 == 0b1100_0000 {
        let b1 = (first & 0b0001_1111) as u32;
        let b2 = (bytes[1] & 0b0011_1111) as u32;
        return (2, (b1 << 6) | b2);
    }

    // Three-byte sequence: 1110xxxx 10xxxxxx 10xxxxxx
    if first & 0b1111_0000 == 0b1110_0000 {
        let b1 = (first & 0b0000_1111) as u32;
        let b2 = (bytes[1] & 0b0011_1111) as u32;
        let b3 = (bytes[2] & 0b0011_1111) as u32;
        return (3, (b1 << 12) | (b2 << 6) | b3);
    }

    // Four-byte sequence: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
    if first & 0b1111_1000 == 0b1111_0000 {
        let b1 = (first & 0b0000_0111) as u32;
        let b2 = (bytes[1] & 0b0011_1111) as u32;
        let b3 = (bytes[2] & 0b0011_1111) as u32;
        let b4 = (bytes[3] & 0b0011_1111) as u32;
        return (4, (b1 << 18) | (b2 << 12) | (b3 << 6) | b4);
    }

    // Input guaranteed valid, so unreachable.
    unreachable!()
}
