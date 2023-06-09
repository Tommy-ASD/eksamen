use per::EncryptedWishListElement;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Scope,
    sql::Thing,
    Surreal,
};
#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SrdbUserElement {
    name: String,
    wishes: Vec<WishListElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WishListGraphConnection<T> {
    bought: Option<Vec<Thing>>,
    #[serde(rename = "in")]
    in_: T,
    out: Thing,
    id: Thing,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SrdbWishList {
    wishes: Vec<WishListGraphConnection<EncryptedWishListElement>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Notification {
    gift_recipient: Thing,
    gift_recipient_name: String,
    id: Thing,
    item_id: Thing,
    name: String,
    notif_recipients: Vec<Thing>,
    price: f64,
    store: String,
}

impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut notif_recipients = String::new();
        for recipient in &self.notif_recipients {
            notif_recipients.push_str(&format!("{}, ", recipient));
        }
        write!(
            f,
            "Gift recipient name: {}\nName: {}\nPrice: {}\nStore: {}\n\n",
            self.gift_recipient_name, self.name, self.price, self.store
        )
    }
}

use crate::per::WishListElement;

pub mod per;
pub mod utils;

pub static DB: Surreal<Client> = Surreal::init();
pub static NS_STR: &'static str = "test";
pub static DB_STR: &'static str = "test";
pub static SC_STR: &'static str = "account";

#[derive(Debug, Serialize)]
struct AuthParams<'a> {
    name: &'a str,
    pass: &'a str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start db as a child process
    // std::process::Command::new("./db.exe")
    //     .arg("start")
    //     .arg("--log")
    //     .arg("none")
    //     .arg("--user")
    //     .arg("root")
    //     .arg("--pass")
    //     .arg("root")
    //     .arg("file://./db")
    //     .spawn()
    //     .expect("Failed to start database");
    DB.connect::<Ws>("localhost:8000")
        .await
        .expect("Failed to connect to database at localhost:8000");
    DB.use_ns(NS_STR).use_db(DB_STR).await?;
    loop {
        let input = crate::input!("Sign in or sign up? (in/up): ");
        match input.as_str() {
            "in" => match signin().await {
                Ok(_) => break,
                Err(e) => println!("Error: {}", e),
            },
            "up" => match signup().await {
                Ok(_) => break,
                Err(e) => println!("Error: {}", e),
            },
            _ => {
                println!("Invalid input")
            }
        }
    }
    after_login().await?;
    Ok(())
}

async fn signup() -> Result<(), Box<dyn std::error::Error>> {
    let name = crate::input!("Username: ");
    let pass = crate::input!("Password: ");
    // Select the namespace/database to use

    // Sign a user in
    DB.signup(Scope {
        namespace: NS_STR,
        database: DB_STR,
        scope: SC_STR,
        params: AuthParams {
            name: &name,
            pass: &pass,
        },
    })
    .await?;
    println!("Signed up as {}", name);
    Ok(())
}

async fn signin() -> Result<(), Box<dyn std::error::Error>> {
    let name = crate::input!("Username: ");
    let pass = crate::input!("Password: ");
    // Select the namespace/database to use

    // Sign a user in
    DB.signin(Scope {
        namespace: NS_STR,
        database: DB_STR,
        scope: SC_STR,
        params: AuthParams {
            name: &name,
            pass: &pass,
        },
    })
    .await?;
    println!("Signed in as {}", name);
    Ok(())
}

async fn after_login() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let input = crate::input!(
            "What do you want to do?
1. Add a wish
2. Read your wishlist
3. Read another user's wishlist
4. View notifications
5. Exit"
        );
        match input.as_str() {
            "1" => match write_wishlist().await {
                Ok(_) => println!("Wishes added"),
                Err(e) => println!("Error: {}", e),
            },
            "2" => match read_wishlist().await {
                Ok(_) => println!("Wishes read"),
                Err(e) => println!("Error: {}", e),
            },
            "3" => read_other_wishlist().await?,
            "4" => view_notifications().await?,
            "5" => break,
            _ => println!("Invalid input"),
        }
    }
    Ok(())
}

async fn write_wishlist() -> Result<(), Box<dyn std::error::Error>> {
    let auth_id = get_auth_id().await?;
    let encryption_key =
        password("Write a decryption key (you and others will need this to read your wishes): ");
    println!("Add wishes: ");
    loop {
        let element = per::WishListElement::new_from_cli();
        let encrypted_element =
            EncryptedWishListElement::from_unencrypted(element, &encryption_key);
        // add to db
        let created: Option<Record> = DB.create("item").content(encrypted_element).await.ok();
        let item_id = match created {
            Some(r) => format!("{}", r.id),
            None => continue,
        };
        let query = format!("RELATE {item_id}->wishes_for->{auth_id};");
        let query = DB.query(query);
        query.await?;

        let input = crate::input!("Add more wishes? (y/n): ");
        if input == "n" {
            break;
        }
    }
    Ok(())
}

async fn read_wishlist() -> Result<(), Box<dyn std::error::Error>> {
    let auth_id = get_auth_id().await?;
    let name = get_auth_name().await?;
    let mut response = DB
        .query(&format!(
            "SELECT <-wishes_for AS wishes FROM {auth_id} FETCH wishes, wishes.in;"
        ))
        .await?;
    let wishes: Vec<SrdbWishList> = response.take(0)?;
    // get the first element
    let encrypted_wishes: Vec<WishListGraphConnection<EncryptedWishListElement>> = wishes
        .first()
        .ok_or("No user with that name")?
        .wishes
        .clone();
    let decryption_key = password("Write the decryption key");
    let wishes: Vec<WishListGraphConnection<WishListElement>> = encrypted_wishes
        .iter()
        .filter(|wish| wish.in_.decrypt(&decryption_key).is_ok())
        .map(|wish| {
            let decrypted = wish.in_.decrypt(&decryption_key).unwrap();
            WishListGraphConnection {
                out: wish.out.clone(),
                in_: decrypted,
                id: wish.id.clone(),
                bought: wish.bought.clone(),
            }
        })
        .collect();
    let decrypted_items: Vec<WishListElement> = wishes.iter().map(|w| w.in_.clone()).collect();
    decrypted_items
        .iter()
        .enumerate()
        .for_each(|(index, wish)| println!("{index}: {wish}", index = index + 1));
    let input = crate::input!("Would you like to delete any of these wishes? (y/n): ");
    if input == "y" {
        let index = crate::input!("Write the index of the wish you want to delete: ");
        let index = index.parse::<usize>()?;
        let wish = wishes.get(index - 1).ok_or("Invalid input")?;
        // delete connection using WishListGraphConnection.id
        let deletion_query = format!("DELETE {wish_id};", wish_id = wish.id);
        let notification_query = format!(
            "UPDATE notification MERGE {{
                \"gift_recipient_name\": \"{name}\",
                \"name\": \"{wish_name}\",
                \"price\": {wish_price},
                \"link\": {wish_link:?},
                \"store\": \"{wish_store}\",
            }} WHERE item_id = {wish_id} AND gift_recipient = {auth_id};",
            wish_name = wish.in_.name,
            wish_price = wish.in_.price,
            wish_link = wish.in_.link,
            wish_store = wish.in_.store,
            wish_id = wish.in_.id.clone().unwrap()
        );
        let query = DB.query(deletion_query).query(notification_query);
        query.await?;
    }
    Ok(())
}

async fn read_other_wishlist() -> Result<(), Box<dyn std::error::Error>> {
    let auth_id = get_auth_id().await?;
    let name = crate::input!("Write the username of the person who's wishlist you want to see: ");
    let mut response = DB
        .query(&format!(
            "SELECT <-wishes_for AS wishes FROM user WHERE name = \"{name}\" FETCH wishes, wishes.in;"
        ))
        .bind(("name", &name))
        .await?;
    let wishes: Vec<SrdbWishList> = response.take(0)?;
    // get the first element
    let encrypted_wishes: Vec<WishListGraphConnection<EncryptedWishListElement>> = wishes
        .first()
        .ok_or("No user with that name")?
        .wishes
        .clone();
    let decryption_key = password("Write the decryption key");
    let mut wishes: Vec<WishListGraphConnection<WishListElement>> = encrypted_wishes
        .iter()
        .filter(|wish| wish.in_.decrypt(&decryption_key).is_ok())
        .map(|wish| {
            let decrypted = wish.in_.decrypt(&decryption_key).unwrap();
            WishListGraphConnection {
                out: wish.out.clone(),
                in_: decrypted,
                id: wish.id.clone(),
                bought: wish.bought.clone(),
            }
        })
        .collect();
    let max_price = crate::input!("What is your max price? (0 for no limit): ");
    let max_price: f64 = max_price.parse()?;
    if max_price != 0.0 {
        wishes = wishes
            .into_iter()
            .filter(|w| w.in_.price <= max_price)
            .collect::<Vec<_>>();
    }
    wishes.sort_by(|a, b| a.in_.price.partial_cmp(&b.in_.price).unwrap());
    wishes.iter().enumerate().for_each(|(i, w)| {
        print!("{i}: {w}", i = i + 1, w = w.in_);
        if let Some(bought) = &w.bought {
            println!(
                " (bought by {bought})",
                bought = bought
                    .iter()
                    .map(|b| format!("{}", b))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            println!();
        }
    });
    let input = crate::input!("Would you like to mark any of these as bought? (y/n)");
    if input == "y" {
        let input = crate::input!("Which one? (number): ");
        let input: usize = input.parse()?;
        let wish = wishes.get(input - 1).ok_or("Invalid input")?;
        let query = format!(
            "UPDATE {wish_id} SET bought += {auth_id};",
            wish_id = wish.id
        );
        let query = DB.query(query);
        query.await?;
    }
    Ok(())
}

async fn view_notifications() -> Result<(), Box<dyn std::error::Error>> {
    let auth_id = get_auth_id().await?;
    let mut response = DB.query("SELECT * FROM notification;").await?;
    let notifications: Vec<Notification> = response.take(0)?;
    let notifications: Vec<Notification> = notifications
        .into_iter()
        .filter(|n| n.notif_recipients.contains(&auth_id))
        .collect();
    for n in notifications {
        println!("{n}");
        // remove self from notification
        let query = format!(
            "UPDATE {id} SET notif_recipients -= {auth_id};",
            auth_id = auth_id,
            id = n.id.clone()
        );
        let query = DB.query(query);
        query.await.unwrap();
    }
    Ok(())
}

fn password<'a>(message: &str) -> Vec<u8> {
    let mut key = crate::input!("{message}").as_bytes().to_vec();
    if key.len() < 16 {
        utils::pad(&mut key);
    }
    let key_ref = key[0..16].to_vec();
    key_ref
}

async fn get_auth_id() -> Result<Thing, Box<dyn std::error::Error>> {
    let auth_query = format!("SELECT id FROM $auth");
    let auth_query = DB.query(auth_query);
    let mut auth_result = auth_query.await?;
    let auth_id = auth_result
        .take::<Vec<HashMap<String, Thing>>>(0)?
        .pop()
        .unwrap()
        .get("id")
        .unwrap()
        .clone();
    Ok(auth_id)
}

async fn get_auth_name() -> Result<String, Box<dyn std::error::Error>> {
    let auth_query = format!("SELECT name FROM $auth");
    let auth_query = DB.query(auth_query);
    let mut auth_result = auth_query.await?;
    let auth_name = auth_result
        .take::<Vec<HashMap<String, String>>>(0)?
        .pop()
        .unwrap()
        .get("name")
        .unwrap()
        .clone();
    let auth_name = format!("{}", auth_name);
    Ok(auth_name)
}
