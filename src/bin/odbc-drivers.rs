use cotton::prelude::*;
use odbc_iter::Odbc;

/// Query ODBC database
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(flatten)]
    logging: LoggingOpt,
}

fn main() -> Result<(), Problem> {
    let args = Cli::from_args();
    init_logger(&args.logging, vec![module_path!(), "odbc_iter"]);

    for driver in Odbc::list_drivers().or_failed_to("list dirvers") {
        println!("{:?}", driver)
    }
    Ok(())
}
