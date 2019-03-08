use cotton::prelude::*;
use odbc_iter::{Odbc, ValueRow};

/// Query ODBC database
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(subcommand)]
    output: Output,
}

#[derive(Debug, StructOpt)]
enum Output {
    /// List ODBC drivers and exit
    #[structopt(name = "list_drivers")]
    ListDrivers,

    /// Print records with Rust Debug output
    #[structopt(name = "debug")]
    Debug {
        #[structopt(name = "connection-string")]
        connection_string: String,

        #[structopt(flatten)]
        query: Query,
    },
}

#[derive(Debug, StructOpt)]
struct Query {
    #[structopt(name = "query")]
    query: Option<String>,

    #[structopt(name = "parameters")]
    parameters: Vec<String>,
}

fn main() -> Result<(), Problem> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!(), "odbc_iter"]);

    let mut odbc = Odbc::new().or_failed_to("initialize ODBC");
    match args.output {
        Output::ListDrivers => {
            for driver in odbc.list_drivers().or_failed_to("list dirvers") {
                println!("{:?}", driver)
            }
            return Ok(());
        }
        Output::Debug {
            connection_string,
            query: Query { query, parameters },
        } => {
            let mut db = odbc.connect(&connection_string).or_failed_to("connect to database");
            let mut db = db.handle();

            let query = query.unwrap_or_else(|| read_stdin());

            let rows = db
                .query_with_parameters::<ValueRow, _>(&query, |q| {
                    parameters
                        .iter()
                        .fold(Ok(q), |q, v| q.and_then(|q| q.bind(v)))
                })
                .or_failed_to("execut query");

            for row in rows {
                println!("{:?}", row.or_failed_to("fetch row data"))
            }
        }
    }

    Ok(())
}
