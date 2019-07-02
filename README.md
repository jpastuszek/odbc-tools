Command line interface tools to query databases with via native ODBC drivers using 'odbc-iter' and 'odbc' crates.

Installation
===========

This package is published on crates.io.

```sh
cargo install odbc-tools
```

Usage example
===========

odbc-query
-----------

Run query and print result set with given formatting.

```sh
odbc-query $CONNECTION_STRING vertical "select * from sys.tables limit 2"
```

Example output:
```
0   id            2001
0   name          schemas
0   schema_id     2000
0   query         NULL
0   type          10
0   system        true
0   commit_action 0
0   access        0
0   temporary     0

1   id            2007
1   name          types
1   schema_id     2000
1   query         NULL
1   type          10
1   system        true
1   commit_action 0
1   access        0
1   temporary     0
```

odbc-drivers
-----------

List installed ODBC drivers.

```sh
odbc-drivers
```

Example output:
```
DriverInfo { description: "Cloudera ODBC Driver for Apache Hive", attributes: {"Driver": "/opt/cloudera/hiveodbc/lib/universal/libclouderahiveodbc.dylib", "Description": "Cloudera ODBC Driver for Apache Hive"} }
DriverInfo { description: "MonetDB", attributes: {"Driver": "/usr/local/Cellar/monetdb/11.31.13/lib/libMonetODBC.so", "Description": "MonetDB Driver", "Setup": "/usr/local/Cellar/monetdb/11.31.13/lib/libMonetODBCs.so"} }
DriverInfo { description: "FreeTDS", attributes: {"Driver": "/usr/local/lib/libtdsodbc.so", "UsageCount": "2", "Description": "FreeTDS unixODBC Driver"} }
DriverInfo { description: "SQL Server", attributes: {"UsageCount": "1", "Description": "FreeTDS unixODBC Driver", "Driver": "/usr/local/lib/libtdsodbc.so"} }
```

odbc-script
-----------

Run SQL script from standard input.

```sh
echo "select * from sys.tables limit 2; select 1 as foo;" | odbc-script $CONNECTION_STRING
```

Example output:
```
  0 0   id            2001
  0 0   name          schemas
  0 0   schema_id     2000
  0 0   query         NULL
  0 0   type          10
  0 0   system        true
  0 0   commit_action 0
  0 0   access        0
  0 0   temporary     0

  0 1   id            2007
  0 1   name          types
  0 1   schema_id     2000
  0 1   query         NULL
  0 1   type          10
  0 1   system        true
  0 1   commit_action 0
  0 1   access        0
  0 1   temporary     0

  1 0   foo 1
```