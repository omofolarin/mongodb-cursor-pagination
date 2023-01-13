#[macro_use]
extern crate bson;
#[macro_use]
extern crate serde;

extern crate mongodb;
extern crate mongodb_cursor_pagination;

use bson::{Bson, Regex};
use mongodb::{options::FindOptions, Client};
use mongodb_cursor_pagination::{find, CursorDirections, FindResult, PaginatedCursor};

#[derive(Debug, Serialize, Deserialize)]
pub struct MyFruit {
    name: String,
    how_many: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let client = Client::with_uri_str("mongodb://localhost:27017/")
            .await
            .expect("Failed to initialize client.");
        let db = client.database("mongodb_cursor_pagination");

        let docs = vec![
            doc! { "name": "Apple", "how_many": 2, "spanish": "Manzana" },
            doc! { "name": "Avocado", "how_many": 5, "spanish": "Aguacate" },
            doc! { "name": "Orange", "how_many": 3, "spanish": "Naranja" },
            doc! { "name": "Bananas", "how_many": 10, "spanish": "Bananas" },
            doc! { "name": "Blackberry", "how_many": 12, "spanish": "Mora" },
            doc! { "name": "Blueberry", "how_many": 10, "spanish": "Arandano" },
            doc! { "name": "Lingonberry", "how_many": 5, "spanish": "Arandano roja" },
            doc! { "name": "Raspberry", "how_many": 5, "spanish": "Frambuesa" },
            doc! { "name": "Strawberry", "how_many": 5, "spanish": "Fresa" },
            doc! { "name": "Grapes", "how_many": 12, "spanish": "Uvas" },
        ];

        // should result in...
        // Orange     | 3
        // Avocado    | 5
        // Apple      | 5
        // Blueberry  | 10
        // Bananas    | 10
        // Grapes     | 12
        // Blackberry | 12

        db.collection("myfruits")
            .insert_many(docs, None)
            .await
            .expect("Unable to insert data");

        // query page 1, 2 at a time
        let mut options = create_options(2, 0);
        // String::from("berry"), String::from("i")
        let filter = doc! { "$or": [
            { "name": Bson::RegularExpression(Regex{ pattern: String::from("berry"), options: String::from("i") }) },
            { "spanish": Bson::RegularExpression(Regex{ pattern: String::from("ana"), options: String::from("i")})},

        ] };
        let paginated_cursor = PaginatedCursor::new(Some(options), None, None);
        let mut find_results: FindResult<MyFruit> =
            find(&paginated_cursor, &db.collection("myfruits"), Some(&filter))
                .await
                .expect("Unable to find data");
        print_details("First page", &find_results);

        // get the second page
        options = create_options(2, 0);
        let mut cursor = find_results.page_info.next_cursor;
        let paginated_cursor =
            PaginatedCursor::new(Some(options), cursor, Some(CursorDirections::Next));
        find_results = find(&paginated_cursor, &db.collection("myfruits"), Some(&filter))
            .await
            .expect("Unable to find data");
        print_details("Second page", &find_results);

        // get previous page
        options = create_options(2, 0);
        cursor = find_results.page_info.start_cursor;
        let paginated_cursor =
            PaginatedCursor::new(Some(options), cursor, Some(CursorDirections::Previous));
        find_results = find(&paginated_cursor, &db.collection("myfruits"), Some(&filter))
            .await
            .expect("Unable to find data");
        print_details("Previous (first) page", &find_results);

        // get the second page again
        options = create_options(2, 0);
        cursor = find_results.page_info.next_cursor;
        let paginated_cursor =
            PaginatedCursor::new(Some(options), cursor, Some(CursorDirections::Next));
        find_results = find(&paginated_cursor, &db.collection("myfruits"), Some(&filter))
            .await
            .expect("Unable to find data");
        print_details("Second page (again)", &find_results);

        // get the third page
        options = create_options(2, 0);
        cursor = find_results.page_info.next_cursor;
        let paginated_cursor =
            PaginatedCursor::new(Some(options), cursor, Some(CursorDirections::Next));
        find_results = find(&paginated_cursor, &db.collection("myfruits"), Some(&filter))
            .await
            .expect("Unable to find data");
        print_details("Third page", &find_results);

        // get previous page
        options = create_options(2, 0);
        cursor = find_results.page_info.start_cursor;
        let paginated_cursor =
            PaginatedCursor::new(Some(options), cursor, Some(CursorDirections::Previous));
        find_results = find(&paginated_cursor, &db.collection("myfruits"), Some(&filter))
            .await
            .expect("Unable to find data");
        print_details("Previous (second) page", &find_results);

        db.collection::<MyFruit>("myfruits")
            .drop(None)
            .await
            .expect("Unable to drop collection");
    }).await.unwrap();

    Ok(())
}

fn create_options(limit: i64, skip: u64) -> FindOptions {
    FindOptions::builder()
        .limit(limit)
        .skip(skip)
        .sort(doc! { "name": 1 })
        .build()
}

fn print_details(name: &str, find_results: &FindResult<MyFruit>) {
    println!(
        "{}:\nitems: {:?}\ntotal: {}\nnext: {:?}\nprevious: {:?}\nhas_previous: {}\nhas_next: {}",
        name,
        find_results.docs,
        find_results.total_docs,
        find_results.page_info.next_cursor,
        find_results.page_info.start_cursor,
        find_results.page_info.has_previous_page,
        find_results.page_info.has_next_page,
    );
    println!("-----------------");
}
