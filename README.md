# Catalyst Funds archive tool

This is the tool intended to create an easy-to-read archive of information
related to Catalyst funds.

## Installation

You need to have the Rust toolchain.

Run `cargo install --git https://github.com/input-output-hk/catalyst-fund-archive-tool.git`.

## Usage

```
catalyst-fund-archive-tool <jormungandr-database> <output-dir>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <jormungandr-database>    The path to the Jormungandr database to dump transactions from
    <output-dir>              CSV output directory
```

### Output format

This tool outputs a set of CSV files. Each file has a name in the following
format: `vote_plan_<vote plan id>.csv`.

The set of columns depends on the type of a vote plan (private or public).

For each type the following information is included:

| Name | Description |
| --- | --- |
| `fragment_id` | The ID of a vote transaction (hex) |
| `caster` | The ID of the account that casted this vote (hex) |
| `proposal` | The number of the proposal this vote was for (number) |
| `time` | The time this vote was casted in the format `epoch.slot` |
| `raw_fragment` | hex-encoded transaction |

### Public vote plans

They carry and additional column named `choice` that carries the number of the
option the account has voted for.

### Private vote plans

_Not yet implemented._
