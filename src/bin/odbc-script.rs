use cotton::prelude::*;
use odbc_iter::{Odbc, ValueRow, AsNullable};

/// Query ODBC database
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(name = "connection-string")]
    connection_string: String,

    #[structopt(name = "query")]
    query: Option<String>,
}

fn main() -> Result<(), Problem> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!(), "odbc_iter"]);

    let mut db = Odbc::connect(&args.connection_string).or_failed_to("connect to database");
    let mut db = db.handle();

    let queries = args.query.unwrap_or_else(|| read_stdin());

    for (q, query) in odbc_iter::split_queries(&queries).or_failed_to("split queries").enumerate() {
        if q > 0 {
            println!();
        }

        let rows = db.query::<ValueRow>(query).or_failed_to("execute query");
        let schema = rows.column_names().to_vec();
        let max_width = schema.iter().map(|c| c.len()).max().unwrap_or(0);
        for (i, row) in rows.or_failed_to("fetch row data").enumerate() {
            if i > 0 {
                println!();
            }
            for (value, column) in row.into_iter().zip(schema.iter()) {
                println!("{:>3} {:<3} {:width$} {}", q, i, column, value.as_nullable(), width = max_width);
            }
        }
    }

    Ok(())
}
