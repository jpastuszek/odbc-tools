use cotton::prelude::*;
use odbc_iter::{Odbc, Handle, ValueRow, Rows, Executed, TryFromRow};
use odbc_iter::value::AsNullable;

/// Query ODBC database
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(name = "connection-string")]
    connection_string: String,

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
        #[structopt(flatten)]
        query: Query,
    },

    /// Print records in vertical form
    #[structopt(name = "vertical")]
    Vertical {
        #[structopt(flatten)]
        query: Query,
    },
}

#[derive(Debug, StructOpt)]
struct Query {
    #[structopt(name = "query")]
    text: Option<String>,

    #[structopt(name = "parameters")]
    parameters: Vec<String>,
}

fn execute<'h, 'c, 'o, T: TryFromRow>(handle: &'h mut Handle<'c, 'o>, query: Query) -> Rows<'h, 'c, T, Executed> {
    let text = query.text.unwrap_or_else(|| read_stdin());
    let parameters = query.parameters;

    handle.query_with_parameters(&text, |q| {
            parameters
                .iter()
                .fold(Ok(q), |q, v| q.and_then(|q| q.bind(v)))
        })
        .or_failed_to("execut query")
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
        _ => ()
    }

    let mut db = odbc.connect(&args.connection_string).or_failed_to("connect to database");
    let mut db = db.handle();

    match args.output {
        Output::Debug { query } => {
            for row in execute::<ValueRow>(&mut db, query).or_failed_to("fetch row data") {
                println!("{:?}", row)
            }
        }
        Output::Vertical { query } => {
            let rows = execute::<ValueRow>(&mut db, query);
            let schema = rows.schema().iter().map(|c| c.name.to_owned()).collect::<Vec<_>>();
            let max_width = schema.iter().map(|c| c.len()).max().unwrap_or(0);

            for (i, row) in rows.or_failed_to("fetch row data").enumerate() {
                if i > 0 {
                    println!();
                }
                for (value, column) in row.into_iter().zip(schema.iter()) {
                    println!("{:<3} {:width$} {}", i, column, value.as_nullable(), width = max_width);
                }
            }
        }
        _ => ()
    }

    Ok(())
}
