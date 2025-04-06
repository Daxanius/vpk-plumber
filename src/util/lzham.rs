//! Helper methods for LZHAM compression and decompression.
//!
//! Uses the [`lzham_alpha_sys`] crate for bindings to the LZHAM alpha library.

use std::mem::size_of;
use std::ptr::null;

use lzham_alpha_sys::{
    lzham_compress_flags_LZHAM_COMP_FLAG_DETERMINISTIC_PARSING,
    lzham_compress_level_LZHAM_COMP_LEVEL_UBER, lzham_compress_memory, lzham_compress_params,
    lzham_compress_status_t_LZHAM_COMP_STATUS_SUCCESS,
    lzham_decompress_flags_LZHAM_DECOMP_FLAG_COMPUTE_ADLER32,
    lzham_decompress_flags_LZHAM_DECOMP_FLAG_OUTPUT_UNBUFFERED, lzham_decompress_memory,
    lzham_decompress_params, lzham_uint32,
};

const TFLZHAM_DICT_SIZE: u32 = 20; // required for compatibility

#[allow(dead_code)]
const TFLZHAM_COMPRESS_PARAMS: lzham_compress_params = lzham_compress_params {
    m_struct_size: size_of::<lzham_compress_params>() as _,
    m_dict_size_log2: TFLZHAM_DICT_SIZE,
    m_compress_flags: lzham_compress_flags_LZHAM_COMP_FLAG_DETERMINISTIC_PARSING as _,
    m_level: lzham_compress_level_LZHAM_COMP_LEVEL_UBER,
    m_max_helper_threads: -1,
    m_cpucache_total_lines: 0,
    m_cpucache_line_size: 0,
    m_num_seed_bytes: 0,
    m_pSeed_bytes: null(),
};

const TFLZHAM_DECOMPRESS_PARAMS: lzham_decompress_params = lzham_decompress_params {
    m_struct_size: size_of::<lzham_decompress_params>() as _,
    m_dict_size_log2: TFLZHAM_DICT_SIZE,
    m_decompress_flags: (lzham_decompress_flags_LZHAM_DECOMP_FLAG_OUTPUT_UNBUFFERED
        | lzham_decompress_flags_LZHAM_DECOMP_FLAG_COMPUTE_ADLER32) as _,
    m_num_seed_bytes: 0,
    m_pSeed_bytes: null(),
};

#[allow(dead_code)]
#[must_use]
pub fn compress(src: &mut [u8]) -> Vec<u8> {
    let max_compressed_size = (1 + src.len()) * 10;
    let mut dst = vec![0; max_compressed_size];
    let mut dst_len = max_compressed_size;

    let mut adler32: lzham_uint32 = 0;

    let status = unsafe {
        lzham_compress_memory(
            &TFLZHAM_COMPRESS_PARAMS,
            dst.as_mut_ptr(),
            &mut dst_len,
            src.as_mut_ptr(),
            max_compressed_size,
            &mut adler32,
        )
    };

    if status != lzham_compress_status_t_LZHAM_COMP_STATUS_SUCCESS {
        eprintln!("LZHAM compression failed with status {status:?}");
        return Vec::new();
    }

    dst.truncate(dst_len);

    dst
}

#[must_use]
pub fn decompress(src: &[u8], mut dst_len: usize) -> Vec<u8> {
    let mut dst = vec![0; dst_len];
    let mut adler32: lzham_uint32 = 0;

    let () = unsafe {
        lzham_decompress_memory(
            &TFLZHAM_DECOMPRESS_PARAMS,
            dst.as_mut_ptr(),
            &mut dst_len,
            src.as_ptr(),
            src.len(),
            &mut adler32,
        );
    };

    dst.truncate(dst_len);

    dst
}
