use crate::{db::models::Item, AppState};
use std::sync::Arc;
use axum::{
    extract::{Extension, Multipart},
    response::Html
};
extern crate serde_json;
use serde_json::from_slice;
use crate::db::postgres::create_item;
use diesel::{pg::PgConnection, result::Error};
use diesel::r2d2::{PooledConnection, ConnectionManager};

pub async fn upload_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input type="file" name="file" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#,
    )
}

pub async fn upload_handler(state: Extension<Arc<AppState>>, mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!(
            "Length of `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
            data.len()
        );

        let items: Vec<Item> = from_slice(&data).unwrap();
        let mut conn = state.db_pool.get().expect("Could not pool a db connector");
        let (validMessages, invalidMessages) = store_items(items, &mut conn);

        let printableInvalidMessages = invalidMessages
        .into_iter()
        .map(|(item, err)| format!("{}: {}", to_string_pretty(&item).unwrap(), err.to_string()))
        .collect::<Vec<_>>();

        use serde_json::to_string_pretty;
        let json1 = to_string_pretty(&validMessages).unwrap();
        println!("{}", json1);
        let json2 = to_string_pretty(&printableInvalidMessages).unwrap();
        println!("{}", json2);
    }        

    fn store_items(items: Vec<Item>, conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> (Vec<Item>, Vec<(Item, Error)>) {
        let mut failed_items = Vec::new();
        let mut successful_items = Vec::new();
    
        for item in items {
            match create_item(conn, &item) {
                Ok(_) => {
                    successful_items.push(item);
                }
                Err(err) => {
                    failed_items.push((item, err));
                }
            }
        }
    
        (successful_items, failed_items)
    }
}