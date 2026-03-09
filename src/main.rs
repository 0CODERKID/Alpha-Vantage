use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select, Input};
use reqwest::Client;
use serde::Deserialize;

const API_KEY: &str = "2M8YK4KITK1A6FZY";
const BASE_URL: &str = "https://www.alphavantage.co/query";

#[derive(Parser)]
#[command(name = "Monitoring the Situation")]
#[command(about = "CLI tool for tracking stocks and crypto using Alpha Vantage")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Stock {
        symbol: String,
    },
    Crypto {
        from_currency: String,
        #[arg(default_value = "USD")]
        to_currency: String,
    },
}

#[derive(Deserialize, Debug)]
struct GlobalQuoteResponse {
    #[serde(rename = "Note")]
    note: Option<String>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
    #[serde(rename = "Global Quote")]
    global_quote: Option<GlobalQuote>,
}

#[derive(Deserialize, Debug)]
struct GlobalQuote {
    #[serde(rename = "01. symbol")]
    symbol: String,
    #[serde(rename = "05. price")]
    price: String,
    #[serde(rename = "09. change")]
    change: String,
    #[serde(rename = "07. latest trading day")]
    latest_trading_day: String,
}

#[derive(Deserialize, Debug)]
struct CryptoResponse {
    #[serde(rename = "Realtime Currency Exchange Rate")]
    exchange_rate: Option<ExchangeRate>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
    #[serde(rename = "Information")]
    information: Option<String>,
    #[serde(rename = "Note")]
    note: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ExchangeRate {
    #[serde(rename = "1. From_Currency Code")]
    from_code: String,
    #[serde(rename = "3. To_Currency Code")]
    to_code: String,
    #[serde(rename = "5. Exchange Rate")]
    exchange_rate: String,
    #[serde(rename = "6. Last Refreshed")]
    last_refreshed: String,
}

async fn get_stock_quote(client: &Client, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}?function=GLOBAL_QUOTE&symbol={}&apikey={}",
        BASE_URL, symbol, API_KEY
    );

    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<GlobalQuoteResponse>()
        .await?;

    if let Some(err) = resp.error_message {
        println!("API Error: {}", err);
        return Ok(());
    }

    if let Some(note) = resp.note {
        println!("API Note: {}", note);
        return Ok(());
    }

    if let Some(quote) = resp.global_quote {
        println!("{:?}", quote);
    } else {
        println!("No data found for symbol {}", symbol);
    }

    Ok(())
}

async fn get_crypto_quote(client: &Client, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}?function=CURRENCY_EXCHANGE_RATE&from_currency={}&to_currency={}&apikey={}",
        BASE_URL, from, to, API_KEY
    );

    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<CryptoResponse>()
        .await?;

    if let Some(err) = resp.error_message {
        println!("API Error: {}", err);
        return Ok(());
    }

    if let Some(info) = resp.information {
        println!("API Info (possibly rate limit): {}", info);
        return Ok(());
    }

    if let Some(note) = resp.note {
        println!("API Note: {}", note);
        return Ok(());
    }

    if let Some(rate) = resp.exchange_rate {
        println!("{:?}", rate);
    } else {
        println!("No data found for {} to {}", from, to);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = Client::new();

    if let Some(command) = &cli.command {
        match command {
            Commands::Stock { symbol } => {
                get_stock_quote(&client, symbol).await?;
            }
            Commands::Crypto { from_currency, to_currency } => {
                get_crypto_quote(&client, from_currency, to_currency).await?;
            }
        }
        return Ok(());
    }

    let theme = ColorfulTheme::default();

    loop {
        let selections = &["Check Stock Quote", "Check Crypto Quote", "Exit"];
        let selection = Select::with_theme(&theme)
            .with_prompt("Monitoring the Situation - Main Menu")
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selection {
            0 => {
                let symbol: String = Input::with_theme(&theme)
                    .with_prompt("Enter stock symbol (e.g. AAPL)")
                    .interact_text()?;
                println!("Fetching stock quote for {}...", symbol);
                get_stock_quote(&client, &symbol).await?;
            }
            1 => {
                let from_currency: String = Input::with_theme(&theme)
                    .with_prompt("Enter from currency code (e.g. BTC)")
                    .interact_text()?;
                let to_currency: String = Input::with_theme(&theme)
                    .with_prompt("Enter target currency")
                    .default("USD".into())
                    .interact_text()?;
                println!(
                    "Fetching exchange rate from {} to {}...",
                    from_currency, to_currency
                );
                get_crypto_quote(&client, &from_currency, &to_currency).await?;
            }
            2 => {
                println!("Exiting Monitoring the Situation. BYE :D !");
                break;
            }
            _ => unreachable!(),
        }
        println!();
    }
    Ok(())
}



