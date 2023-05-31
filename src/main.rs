use std::fs;

pub mod per;
pub mod utils;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    loop {
        let write = crate::input!(
            "
1. Create new wishlist
2. Read wishlist
"
        );
        match write.as_str() {
            "1" => {
                let key = password();
                let filepath = crate::input!("File path: ");
                let padding = write_wishlist(&key, &format!("./wishlists/{}", &filepath));
                write_padding(&filepath, padding);
            }
            "2" => {
                let key = password();
                let filepath = crate::input!("File path: ");
                let wishlist = read_wishlist(&key, &format!("./wishlists/{}", &filepath));
                wishlist.iter().for_each(|element| {
                    println!("{}", element);
                });
            }
            _ => {
                println!("Invalid input");
            }
        }
    }
}

fn write_wishlist(key: &[u8], file_path: &str) -> u8 {
    println!("Add wishes: ");
    let mut wish_list = vec![];
    loop {
        let element = per::WishListElement::new_from_cli();
        println!("{}", element);
        wish_list.push(element);
        let input = crate::input!("Add more wishes? (y/n): ");
        if input == "n" {
            break;
        }
    }
    let stringified = serde_json::to_string(&wish_list).unwrap();
    let bytes = stringified.as_bytes();
    let (encrypted_bytes, padding) = utils::encrypt(key, bytes);
    fs::write(file_path, encrypted_bytes).unwrap();
    padding
}

fn read_wishlist(key: &[u8], file_path: &str) -> Vec<per::WishListElement> {
    let padding = read_padding(file_path);
    let encrypted_bytes = fs::read(file_path).unwrap();
    let decrypted_bytes = utils::decrypt(key, &encrypted_bytes, 0);
    let decrypted_bytes = &decrypted_bytes[0..decrypted_bytes.len() - padding as usize];
    let stringified = String::from_utf8(decrypted_bytes.to_vec()).unwrap();
    println!("{}", stringified);
    let wish_list: Vec<per::WishListElement> = serde_json::from_str(&stringified).unwrap();
    wish_list
}

fn write_padding(key: &str, padding: u8) {
    // read padding_manager.json
    let padding_manager = match std::fs::read_to_string("padding_manager.json") {
        Ok(padding_manager) => padding_manager,
        Err(_) => {
            let padding_manager = serde_json::Value::Object(serde_json::Map::new());
            serde_json::to_string_pretty(&padding_manager).unwrap()
        }
    };
    let mut padding_manager: serde_json::Value = serde_json::from_str(&padding_manager).unwrap();
    // add padding to padding_manager
    padding_manager[key] = serde_json::Value::from(padding);
    // write padding_manager.json
    let padding_manager = serde_json::to_string_pretty(&padding_manager).unwrap();
    std::fs::write("padding_manager.json", padding_manager).unwrap();
}

fn read_padding(key: &str) -> u8 {
    // read padding_manager.json
    let padding_manager = std::fs::read_to_string("padding_manager.json").unwrap();
    let padding_manager: serde_json::Value = serde_json::from_str(&padding_manager).unwrap();
    // get padding from padding_manager
    let padding = match padding_manager.get(key) {
        Some(serde_json::Value::Number(padding)) => padding.as_u64().unwrap() as u8,
        _ => 0,
    };
    padding as u8
}

fn password<'a>() -> Vec<u8> {
    let mut key = crate::input!("Password (16 characters): ")
        .as_bytes()
        .to_vec();
    if key.len() < 16 {
        utils::pad(&mut key);
    }
    let key_ref = key[0..16].to_vec();
    key_ref
}
