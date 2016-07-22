use std::mem;
use std::io::{Error, ErrorKind, Result};
use super::liblz4::*;
use libc::{c_int};
use byteorder::{ByteOrder, NetworkEndian};

const MAX_INPUT_SIZE: usize = 0x7E000000;

pub fn decompress(src: &[u8]) -> Result<Vec<u8>> {
    let header_len = mem::size_of::<u32>();

    if src.len() < header_len {
        return Err(Error::new(ErrorKind::UnexpectedEof, "input too short"));
    }

    let uncompressed_size = NetworkEndian::read_u32(src) as usize;

    if uncompressed_size > MAX_INPUT_SIZE {
        return Err(Error::new(ErrorKind::InvalidData, "invalid size in header"));
    }

    let mut dest = Vec::with_capacity(uncompressed_size);

    unsafe { dest.set_len(uncompressed_size) };

    try!(check_error(unsafe {
        LZ4_decompress_safe(src[header_len..].as_ptr(),
                            dest.as_mut_ptr(),
                            (src.len() - header_len) as c_int,
                            uncompressed_size as c_int) as usize
    }));

    Ok(dest)
}

pub enum CompressMode {
    Default,
    Fast,
    HighCompression
}

fn compress_impl(src: &[u8], mode: CompressMode) -> Result<Vec<u8>> {
    let header_len = mem::size_of::<u32>();

    let compressed_size = unsafe { LZ4_compressBound(src.len() as i32) as usize };

    let mut dest = Vec::with_capacity(header_len + compressed_size);

    unsafe { dest.set_len(header_len + compressed_size) };

    NetworkEndian::write_u32(&mut dest, src.len() as u32);

    let actual_size = match mode {
        _ => unsafe {
            LZ4_compress_default(src.as_ptr(),
                                 dest[header_len..].as_mut_ptr(),
                                 src.len() as c_int,
                                 compressed_size as c_int) as usize
        }
    };

    if actual_size == 0 {
        return Err(Error::new(ErrorKind::Other, "unable to compress"));
    }

    dest.truncate(header_len + actual_size);

    Ok(dest)
}

pub fn compress(src: &[u8]) -> Result<Vec<u8>> {
    compress_impl(src, CompressMode::Default)
}

pub fn compress_fast(src: &[u8]) -> Result<Vec<u8>> {
    compress_impl(src, CompressMode::Fast)
}

#[cfg(test)]
mod test {
    extern crate rand;
    use self::rand::Rng;
    use super::{compress, decompress};

    #[test]
    fn test_decoder_random() {
        let mut expected = vec![0; 512];

        rand::thread_rng().fill_bytes(&mut expected);

        println!("expected: {:?}", &expected);
        let compressed = compress(&expected).unwrap();
        println!("compressed: {:?}", compressed);

        let decompressed = decompress(&compressed).unwrap();

        assert_eq!(expected, decompressed);
    }
}