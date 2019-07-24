use cotton::prelude::*;
use odbc_iter::{Odbc, Handle, ValueRow, ResultSet, Executed, TryFromRow, AsNullable};
use odbc_avro::{ToAvroSchema, WriteAvro};
use serde_json;

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

    /// Print records in JSON array form
    #[structopt(name = "json-array")]
    JsonArray {
        #[structopt(flatten)]
        query: Query,
    },

    /// Serilize to Avro binary format one "record" per row
    #[structopt(name = "avro-record")]
    AvroRecord {
        /// Print Avro schema only
        #[structopt(long = "show-schema")]
        show_schema: bool,

        /// Schema name
        #[structopt(long = "schema-name", default_value = "result_set")]
        schema_name: String,

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

impl Query {
    fn execute<'h, 'c, T: TryFromRow>(self, handle: &'h mut Handle<'c>) -> ResultSet<'h, 'c, T, Executed> {
        let text = self.text.unwrap_or_else(|| read_stdin());
        let parameters = self.parameters;

        handle.query_with_parameters(&text, |q| {
                parameters
                    .iter()
                    .fold(Ok(q), |q, v| q.and_then(|q| q.bind(v)))
            })
            .or_failed_to("execut query")
    }
}

fn main() -> Result<(), Problem> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!(), "odbc_iter"]);

    let mut db = Odbc::connect(&args.connection_string).or_failed_to("connect to database");
    let mut db = db.handle();

    match args.output {
        Output::Debug { query } => {
            for row in query.execute::<ValueRow>(&mut db).or_failed_to("fetch row data") {
                println!("{:?}", row)
            }
        }
        Output::Vertical { query } => {
            let rows = query.execute::<ValueRow>(&mut db);
            let column_names = rows.schema().iter().map(|s| s.name.clone()).collect::<Vec<_>>();
            let max_width = column_names.iter().map(|c| c.len()).max().unwrap_or(0);

            for (i, row) in rows.or_failed_to("fetch row data").enumerate() {
                if i > 0 {
                    println!();
                }
                for (value, column) in row.into_iter().zip(column_names.iter()) {
                    println!("{:<3} {:width$} {}", i, column, value.as_nullable(), width = max_width);
                }
            }
        }
        Output::JsonArray { query } => {
            for row in query.execute::<ValueRow>(&mut db).or_failed_to("fetch row data") {
                println!("{}", serde_json::to_string(&row).or_failed_to("serialize JSON"))
            }
        }
        Output::AvroRecord { show_schema: true, schema_name, query } => {
            println!("{}", query.execute::<ValueRow>(&mut db).schema().to_avro_schema(&schema_name).or_failed_to("show Avro schema").canonical_form());
        }
        Output::AvroRecord { show_schema: false, schema_name, query } => {
            query.execute(&mut db).write_avro(&mut stdout(), odbc_avro::Codec::Deflate, &schema_name).or_failed_to("write query result set as Avro data");
        }
    }

    Ok(())
}
