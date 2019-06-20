use cotton::prelude::*;
use odbc_iter::Odbc;

/// Query ODBC database
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,

    #[structopt(name = "connection-string")]
    connection_string: String,
}

fn main() -> Result<(), Problem> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!(), "odbc_iter"]);

    let mut odbc = Odbc::new().or_failed_to("initialize ODBC");
    for driver in odbc.list_drivers().or_failed_to("list dirvers") {
        println!("{:?}", driver)
    }
    Ok(())
}
