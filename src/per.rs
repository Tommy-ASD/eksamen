use std::fmt::Display;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use url::Url;

use crate::utils::encrypt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WishListElement {
    pub id: Option<Thing>,
    pub name: String,
    pub price: f64,
    pub store: String,
    pub link: Option<Url>,
}

impl WishListElement {
    pub fn new(name: String, price: f64, store: String, link: Option<Url>) -> Self {
        Self {
            id: None,
            name,
            price,
            store,
            link,
        }
    }
    pub fn new_from_cli() -> Self {
        let name = crate::input!("Navn: ");
        let mut price = None;
        // this loop will run until price is Some(f64)
        // in other words, until the user inputs a valid f64
        while price.is_none() {
            let mut price_str = crate::input!("Pris: ");
            if !price_str.contains('.') {
                price_str.push_str(".0");
            }
            price = price_str.parse::<f64>().ok();
            // if price is Ok, break the loop
            if let Some(_) = price {
                break;
            } else {
                println!("Pris må være et tall!");
            }
        }
        let price = price.unwrap();
        let store = crate::input!("Butikk: ");
        let add_link = crate::input!("Legg til link? (y/n): ");
        let link = if add_link == "y" {
            let mut link = crate::input!("Link: ");
            if !link.starts_with("http") {
                link = format!("https://{}", link);
            }
            Url::parse(&link).ok()
        } else {
            None
        };
        Self::new(name, price, store, link)
    }
    fn from_encrypted(
        encrypted: &EncryptedWishListElement,
        decryption_key: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let name = String::from_utf8(crate::utils::decrypt(
            decryption_key,
            &encrypted.name,
            encrypted.paddings.name,
        ))?;
        let price = String::from_utf8(crate::utils::decrypt(
            decryption_key,
            &encrypted.price,
            encrypted.paddings.price,
        ))?
        .parse::<f64>()?;
        let store = String::from_utf8(crate::utils::decrypt(
            decryption_key,
            &encrypted.store,
            encrypted.paddings.store,
        ))?;
        let link = if let Some(link) = &encrypted.link {
            Some(Url::parse(&String::from_utf8(crate::utils::decrypt(
                decryption_key,
                &link,
                encrypted.paddings.link.unwrap(),
            ))?)?)
        } else {
            None
        };
        Ok(Self {
            id: encrypted.id.clone(),
            name,
            price,
            store,
            link,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Paddings {
    pub name: u8,
    pub price: u8,
    pub store: u8,
    pub link: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedWishListElement {
    pub id: Option<Thing>,
    pub name: Vec<u8>,
    pub price: Vec<u8>,
    pub store: Vec<u8>,
    pub link: Option<Vec<u8>>,
    pub paddings: Paddings,
}

impl EncryptedWishListElement {
    pub fn from_unencrypted(wish_list_element: WishListElement, encryption_key: &[u8]) -> Self {
        let (enc_name, name_padding) = encrypt(encryption_key, wish_list_element.name.as_bytes());
        let (enc_price, price_padding) = encrypt(
            encryption_key,
            wish_list_element.price.to_string().as_bytes(),
        );
        let (enc_store, store_padding) =
            encrypt(encryption_key, wish_list_element.store.as_bytes());
        let (enc_link, link_padding) = if let Some(link) = wish_list_element.link {
            let (enc_link, link_padding) = encrypt(encryption_key, link.to_string().as_bytes());
            (Some(enc_link), Some(link_padding))
        } else {
            (None, None)
        };
        Self {
            id: None,
            name: enc_name,
            price: enc_price,
            store: enc_store,
            link: enc_link,
            paddings: Paddings {
                name: name_padding,
                price: price_padding,
                store: store_padding,
                link: link_padding,
            },
        }
    }
    pub fn decrypt(
        &self,
        decryption_key: &[u8],
    ) -> Result<WishListElement, Box<dyn std::error::Error>> {
        WishListElement::from_encrypted(self, decryption_key)
    }
}

impl Display for WishListElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let link = if let Some(link) = &self.link {
            link.to_string()
        } else {
            String::from("Ingen link")
        };
        write!(
            f,
            "
Navn: {}
    Pris: {}
    Butikk: {}
    Link: {}",
            self.name, self.price, self.store, link
        )
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::per::{EncryptedWishListElement, WishListElement};

    #[test]
    fn test_encrypt_unencrypt() {
        let key = b"12345678901234567890123456789012";
        let wish = WishListElement::new(
            String::from("Test"),
            123.0,
            String::from("Test"),
            Some(Url::parse("https://test.com").unwrap()),
        );
        let encrypted = EncryptedWishListElement::from_unencrypted(wish, key);
        let decrypted = WishListElement::from_encrypted(&encrypted, key).unwrap();
        assert_eq!(decrypted.name, String::from("Test"));
        assert_eq!(decrypted.price, 123.0);
        assert_eq!(decrypted.store, String::from("Test"));
        assert_eq!(
            decrypted.link,
            Some(Url::parse("https://test.com").unwrap())
        );
    }
}
