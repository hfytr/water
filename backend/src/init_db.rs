use sqlx::{Error, PgPool};
use std::env::var;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let table_name = "user_table";
    let cols = vec![
        (
            "user_id".to_string(),
            "int not null primary key".to_string(),
        ),
        ("pass_hash".to_string(), "bit(32)".to_string()),
    ];
    let db_user = var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());
    let db_name = var("POSTGRES_DB").unwrap_or_else(|_| "db".to_string());
    let db_port = var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_host = var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );

    let pool = PgPool::connect(&database_url).await?;

    dbg!("hi");
    dbg!(
        sqlx::query_scalar::<_, Option<bool>>(
            "select exists (
            select 1
            from pg_tables
            where schemaname = 'public'
            and tablename = $1
        )",
        )
        .bind(&table_name)
        .fetch_one(&pool)
        .await?
    );
    let query = format!(
        "select exists (
            select 1
            from pg_tables
            where schemaname = 'public'
            and tablename = '{}'
        )",
        table_name
    );
    dbg!(
        sqlx::query_scalar::<_, Option<bool>>(&query)
            .fetch_one(&pool)
            .await?
    );
    if !sqlx::query_scalar::<_, Option<bool>>(
        "select exists (
            select 1
            from pg_tables
            where schemaname = 'public'
            and tablename = $1
        )",
    )
    .bind(&table_name)
    .fetch_one(&pool)
    .await?
    .unwrap_or(false)
    {
        let query = format!(
            "create table {} ({})",
            table_name,
            cols.iter()
                .map(|(col_name, col_desc)| format!("{} {}", col_name, col_desc))
                .skip(1)
                .fold(format!("{} {}", cols[0].0, cols[0].1), |acc, elem| {
                    format!("{},\n{}", acc, elem)
                }),
        );
        dbg!("table dne");
        sqlx::query(&query).execute(&pool).await?;
        Ok(())
    } else {
        for (col_name, col_desc) in cols {
            dbg!("table e");
            let query = format!(
                "
                    select {}
                    from information_schema.columns
                    where table_name='{}' and column_name='{}'
                ",
                col_name, table_name, col_name
            );
            dbg!(&query);
            if sqlx::query_scalar::<_, Option<bool>>(&query)
                .fetch_one(&pool)
                .await?
                .unwrap_or(false)
            {
                continue;
            }
            let query = format!("alter table {} add {} {}", table_name, col_name, col_desc);
            sqlx::query(&query).execute(&pool).await?;
        }
        Ok(())
    }
}
