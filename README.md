# Sharesight price updater tool

Command-line tool to add price data for custom investments that aren't supported directly by sharesight.

* Uses [sharesight v3 API](https://portfolio.sharesight.com/api/3/overview) – you must ask their support for an API (v3) account.
* Requires your `CLIENT_SECRET` and `CLIENT_ID` to by set as environment variables.
* Written in rust.


# Installation

* Install rust, and compile this project with `cargo build`.
* To run directly from this directory, use `cargo run` in place of the `sharesight` command in the below examples


## Usage

```
$ sharesight --help

sharesight
Custom price updater for sharesight portfolios

USAGE:
    sharesight <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    help      Print this message or the help of the given subcommand(s)
    list      Lists custom investments, showing their name and IDs
    update    Adds a new price at a specific date for the given investment
    scrape    Try and scrape the price of a supported fund from the web
```

### Listing your custom investments

(I only have one)

```
$ sharesight list
2256461	IE00B3X1NT05	Vanguard Global Small-Cap Index Fund GBP Acc
```

### Adding a new price

```
$ sharesight update --help

sharesight-update
Adds a new price at a specific date for the given investment

USAGE:
    sharesight update [OPTIONS] <investment> <date> <price>

ARGS:
    <investment>    The custom investment code, or internal sharesight ID if you pass --use-id
    <date>          The date, formatted like YYYY-MM-DD
    <price>         The price at this date

OPTIONS:
    -h, --help      Print help information
        --use-id    Identify the investment using the internal sharesight ID, not your custom code
```

Eg:

```
$ sharesight update IE00B3X1NT05 2022-01-21 375.86
```

or the same thing, using the internal sharesight id from the `--list` command:

```
$ sharesight update --use-id 2256461 2022-01-21 375.86
```
**NB this update request will fail if there is already a custom price set for the date you provide. Click the "Manage Prices" in the "Instrument Detail" box on the sharesight portfolio page for your investmentm to see what prices you have added, and delete one if you want to try re-adding with this tool.**

## Custom price-scraper

There is exactly one built-in price fetcher which reads from the vanguard website for the above fund, because I need it for my portfolio.
It fetches the navPrice using a json request made by [this vanguard fund page](https://www.vanguardinvestor.co.uk/investments/vanguard-global-small-cap-index-fund-gbp-acc/price-performance).


```
$ sharesight scrape --list
IE00B3X1NT05	Vanguard Global Small-Cap Index Fund

$ sharesight scrape IE00B3X1NT05
IE00B3X1NT05	2022-01-21	375.8605
```

...and assuming you have a custom investment in your sharesight portfolio with the same code, you can say this to update the price:

```
$ sharesight update $(sharesight scrape IE00B3X1NT05)
```

