use std::error::Error;
use sqlx::{ Pool, Postgres,FromRow, Row};
use dotenv::dotenv;
use std::env;
use uuid::Uuid;
use futures::TryStreamExt;

// Tutorial for SQLX: 
//https://www.youtube.com/watch?v=TCERYbgvbq0

#[derive(Debug, FromRow)]
struct Book {
    title: String,
    author: String,
    isbn: String,
}


async fn create(book: &Book, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "INSERT INTO book (title, author, isbn) VALUES ($1, $2, $3)";

    sqlx::query(query)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .execute(pool)
        .await?;

    Ok(())
}   

async fn update_book(book: &Book, isbn: &str, pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    let query = "UPDATE book SET title = %1, author = $2 WHERE isbn = $3";

    sqlx::query(query)
    .bind(&book.title)
    .bind(&book.author)
    .bind(&book.isbn)
    .execute(pool)
    .await?;

    Ok(())
}

async fn read_book(conn: &sqlx::PgPool) -> Result<Book, Box<dyn Error>> {
    let query = "SELECT title, author, isbn FROM book";

    let row = sqlx::query(query)
                                                .fetch_one(conn)
                                                .await?;

    // there is also .fetch_all(), fetch_optional, and fetch


    let book = Book{
        title: row.get("title"),
        author: row.get("author"),
        isbn: row.get("isbn"),
    };

    Ok(book)
}


async fn read_books_turbo_fish(conn: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = "SELECT title, author, isbn FROM book";

    // handles conversion for us with Turbo Fish
    let query = sqlx::query_as::<_, Book>(q);
    
    // there is also .fetch_all(), fetch_optional, and fetch
    let books = query.fetch_all(conn).await?;

    Ok(books)

}

// creating both a Book and an Author will require some finess



async fn read_all_books(conn: &sqlx::PgPool) -> Result<Vec<Book>, Box<dyn Error>> {
    let q = "SELECT title, author, isbn from book";

    let query = sqlx::query(q);

    // retrieve the rows 
    let mut rows = query.fetch(conn);
    
    let mut books = vec![];
    
    while let Some(row) = rows.try_next().await? {
        // add the books to the array
        books.push(Book {
            title: row.get("title"),
            author: row.get("author"),
            isbn: row.get("isbn"),
        })
    }

    Ok(books)

}

#[tokio::main]
async fn main()  -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL is missing from configuration");
    let conn = Pool::<Postgres>::connect(&url).await?;

    // let's run a migration
    sqlx::migrate!("./migrations").run(&conn).await?;

    // let res = sqlx::query("SELECT 69 + 420 as sum")
    //         .fetch_one(&conn)
    //         .await?;

    // let sum: i32 = res.get("sum");
    // println!("the best numbers 69 and 420 added are {}", sum);

    // make a uuid 
    // create a new book
    let book = Book{
        title:"Reverberation".to_string(),
        author: "Keith Blanchard".to_string(),
        isbn: "9781419761898".to_string(),
    };

    create(&book, &conn).await?;

    Ok(())
}
