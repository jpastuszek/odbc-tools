use cotton::prelude::*;
use odbc_iter::{Odbc, Handle, Configuration, ValueRow, ResultSet, Executed, TryFromRow, AsNullable, ColumnType};
use odbc_avro::{AvroRowRecord, AvroResultSet, AvroConfiguration, TimestampFormat, ReformatJson, AvroConfigurationBuilder};
use serde_json;
use structopt::StructOpt;

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
    /// Print schema of the query
    #[structopt(name = "schema")]
    Schema {
        #[structopt(flatten)]
        query: Query,
    },

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

    /// Serilize to Avro "Object Container File" format one "record" per row
    #[structopt(name = "avro-record")]
    AvroRecord {
        /// Print Avro schema only
        #[structopt(long = "show-schema")]
        show_schema: bool,

        /// Use deflate compression
        #[structopt(long = "deflate")]
        deflate: bool,

        /// Parse and format JSON
        #[structopt(long = "reformat-json")]
        reformat_json: bool,

        /// If reformatting JSON use pretty format
        #[structopt(long = "reformat-json-pretty")]
        reformat_json_pretty: bool,

        /// Represent timestamp as number of millisceconds since epoch instead of string
        #[structopt(long = "timestamp-millis")]
        timestamp_millis: bool,

        /// Schema name
        #[structopt(long = "schema-name", default_value = "result_set")]
        schema_name: String,

        #[structopt(flatten)]
        query: Query,
    },
}

fn minus_none(v: String) -> Option<String> {
    if v == "-" {
        None
    } else {
        Some(v)
    }
}

#[derive(Debug, StructOpt)]
struct Query {
    /// Query text or '-' to force to read query from stdin
    #[structopt(name = "query")]
    text: Option<String>,

    /// Parameter values to bind to query
    #[structopt(name = "parameters")]
    parameters: Vec<String>,
}

impl Query {
    fn execute<'h, 'c, T: TryFromRow<C>, C: Configuration + 'h>(self, handle: &'h mut Handle<'c, C>) -> ResultSet<'h, 'c, T, Executed, C> {
        let text = self.text.and_then(minus_none).unwrap_or_else(|| read_stdin());
        let parameters = self.parameters;

        handle.query_with_parameters(&text, |q| {
                parameters
                    .iter()
                    .fold(Ok(q), |q, v| q.and_then(|q| q.bind(v)))
            })
            .or_failed_to("execut query")
    }

    fn schema<'h, 'c, C: Configuration + 'h>(self, handle: &'h mut Handle<'c, C>) -> Vec<ColumnType> {
        let text = self.text.and_then(minus_none).unwrap_or_else(|| read_stdin());

        handle.prepare(&text)
            .or_failed_to("prepare query")
            .schema()
            .or_failed_to("get prepared statement schema")
    }
}

fn make_avro_configuration(reformat_json: bool, reformat_json_pretty: bool, timestamp_millis: bool) -> AvroConfiguration {
    AvroConfigurationBuilder::default()
        .with_reformat_json(match (reformat_json, reformat_json_pretty) {
            (true, true) => Some(ReformatJson::Pretty),
            (true, false) => Some(ReformatJson::Compact),
            (false, _) => None,
        })
        .with_timestamp_format(if timestamp_millis {
            TimestampFormat::MillisecondsSinceEpoch
        } else {
            TimestampFormat::DefaultString
        })
        .build()
}

fn main() -> Result<(), Problem> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!(), "odbc_iter"]);

    let mut db = Odbc::connect(&args.connection_string).or_failed_to("connect to database");
    let mut db = db.handle();

    match args.output {
        Output::Schema { query } => {
            println!("column name\tdatum type\tODBC type\tnullable");
            for column in query.schema(&mut db) {
                println!("{name}\t{datum_type:?}\t{odbc_type:?}\t{nullable}",
                         name = column.name,
                         datum_type = column.datum_type,
                         odbc_type = column.odbc_type,
                         nullable = column.nullable);
            }
        }
        Output::Debug { query } => {
            for row in query.execute::<ValueRow, _>(&mut db).or_failed_to("fetch row data") {
                println!("{:?}", row)
            }
        }
        Output::Vertical { query } => {
            let rows = query.execute::<ValueRow, _>(&mut db);
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
            for row in query.execute::<ValueRow, _>(&mut db).or_failed_to("fetch row data") {
                println!("{}", serde_json::to_string(&row).or_failed_to("serialize JSON"))
            }
        }
        Output::AvroRecord { show_schema: true, schema_name, query, reformat_json, reformat_json_pretty, timestamp_millis, .. } => {
            let mut db = db.with_configuration(make_avro_configuration(reformat_json, reformat_json_pretty, timestamp_millis));
            println!("{}", query.execute::<AvroRowRecord, _>(&mut db).avro_schema(&schema_name).or_failed_to("show Avro schema").canonical_form());
        }
        Output::AvroRecord { show_schema: false, schema_name, query, deflate, reformat_json, reformat_json_pretty, timestamp_millis } => {
            let mut db = db.with_configuration(make_avro_configuration(reformat_json, reformat_json_pretty, timestamp_millis));
            let codec = if deflate {
                odbc_avro::Codec::Deflate
            } else {
                odbc_avro::Codec::Null
            };
            query.execute(&mut db).write_avro(&mut stdout(), codec, &schema_name).or_failed_to("write query result set as Avro data");
        }
    }

    Ok(())
}
