use aes::{
    cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128,
};

pub fn encrypt(key: &[u8], bytes: &[u8]) -> (Vec<u8>, u8) {
    let cipher: Aes128 = Aes128::new(GenericArray::from_slice(key));
    let mut block = [0u8; 16];
    let mut ciphertext = Vec::new();

    let padding_bytes = 16 - (bytes.len() % 16) as u8;
    let padded_bytes = [bytes, &[padding_bytes; 16][..padding_bytes as usize]].concat();

    for chunk in padded_bytes.chunks(16) {
        block[..chunk.len()].copy_from_slice(chunk);
        cipher.encrypt_block(GenericArray::from_mut_slice(&mut block));
        ciphertext.extend_from_slice(&block);
    }

    (ciphertext, padding_bytes)
}

pub fn decrypt(key: &[u8], encrypted_bytes: &[u8], padding: u8) -> Vec<u8> {
    let cipher = Aes128::new(GenericArray::from_slice(key));
    let mut block = [0u8; 16];
    let mut decrypted_bytes = Vec::new();

    for (i, chunk) in encrypted_bytes.chunks(16).enumerate() {
        // if on last block, remove padding
        if i == encrypted_bytes.len() / 16 - 1 {
            let mut block_copy = [0u8; 16];
            block_copy.copy_from_slice(chunk);
            cipher.decrypt_block(GenericArray::from_mut_slice(&mut block_copy));
            decrypted_bytes.extend_from_slice(&block_copy[..16 - padding as usize]);
        } else {
            block.copy_from_slice(chunk);
            cipher.decrypt_block(GenericArray::from_mut_slice(&mut block));
            decrypted_bytes.extend_from_slice(&block);
        }
    }

    decrypted_bytes
}

pub fn pad(bytes: &mut Vec<u8>) -> u8 {
    let padding_bytes = 16 - (bytes.len() % 16) as u8;
    let padded_bytes = [bytes, &[padding_bytes; 16][..padding_bytes as usize]].concat();
    *bytes = padded_bytes;
    padding_bytes
}

pub fn unpad(bytes: &[u8], padding: u8) -> Result<Vec<u8>, &'static str> {
    if padding == 0 || padding > 16 {
        return Err("Invalid padding byte");
    }
    let padding_size = padding as usize;
    if padding_size > bytes.len() {
        return Err("Padding size larger than input size");
    }
    let padding = &bytes[bytes.len() - padding_size..];
    if padding.iter().any(|&b| b != padding[0]) {
        return Err("Invalid padding bytes");
    }
    Ok(bytes[..bytes.len() - padding_size].to_vec())
}

// python-like input macro
#[macro_export]
macro_rules! input {
    () => {
        {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        }
    };
    // if you want to specify a prompt
    ($prompt:expr) => {
        {
            println!($prompt);
            crate::input!()
        }
    };
    // if you want to specify multiple prompts
    ($($prompt:expr),*) => {
        {
            $(
                println!($prompt);
            )*
            crate::input!()
        }
    };
}
