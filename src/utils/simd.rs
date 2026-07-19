use eframe::epaint::Color32;
use std::arch::x86_64::*;

pub(crate) fn simd_multiply_color(color: &Color32, k: f32) -> Color32 {

    // 1. Load the color components into an SSE register (128-bit vector).
    let color_vec_raw = unsafe {
        _mm_set_ps(
            color.r() as f32,
            color.g() as f32,
            color.b() as f32,
            0.0, // Padding
        )
    };

    // 2. Broadcast the scalar 'k' into a SIMD vector.
    let k_vec = unsafe {
        _mm_set1_ps(k)
    };

    // 3. Perform the vectorized multiplication.
    let result_vec = unsafe {
        _mm_mul_ps(color_vec_raw, k_vec)
    };

    // 4. Extract the results and cast to u8.
    let b = unsafe { f32::from_bits(_mm_extract_ps::<1>(result_vec) as u32) } as u8;
    let g = unsafe { f32::from_bits(_mm_extract_ps::<2>(result_vec) as u32) } as u8;
    let r = unsafe { f32::from_bits(_mm_extract_ps::<3>(result_vec) as u32) } as u8;

    // 5. Assemble data to Color32
    Color32::from_rgb(r, g, b)
}

pub(crate) fn simd_add_color(color1: &Color32, color2: &Color32) -> Color32 {

    // 1. Load the colors into SIMD vectors (XMM registers).
    let color_vec1 = unsafe {
        _mm_set_ps(
            color1.r() as f32,
            color1.g() as f32,
            color1.b() as f32,
            0.0, // Padding
        )
    };
    let color_vec2 = unsafe {
        _mm_set_ps(
            color2.r() as f32,
            color2.g() as f32,
            color2.b() as f32,
            0.0, // Padding
        )
    };

    // 2. Perform the vectorized addition.
    let result_vec = unsafe {
        _mm_add_ps(color_vec1, color_vec2)
    };

    // 3. Extract the results and cast to u8.
    let b = unsafe { f32::from_bits(_mm_extract_ps::<1>(result_vec) as u32) } as u8;
    let g = unsafe { f32::from_bits(_mm_extract_ps::<2>(result_vec) as u32) } as u8;
    let r = unsafe { f32::from_bits(_mm_extract_ps::<3>(result_vec) as u32) } as u8;

    // 4. Assemble data to Color32
    Color32::from_rgb(r, g, b)
}