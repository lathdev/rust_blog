use std::env;
use sqlx::postgres::PgPoolOptions;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let args: Vec<String> = env::args().collect();
    let inserter;

    match File::open(&args[2]) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            inserter = content;
        },
        Err(error) => {
            println!("{}", error);
            panic!("Could not insert into db");
        },
    }

    let pool = PgPoolOptions::new()
    .max_connections(3)
    .connect("postgres://myuser:mypass@localhost:5432/mydb")
    .await
    .expect("Couldn't create pool");

    let _row: (i32,) = sqlx::query_as("insert into myposts (post_title, post_body) values ($1, $2) returning post_id")
        .bind(&args[1])
        .bind(inserter)
        .fetch_one(&pool)
        .await?;
    Ok(())
}
