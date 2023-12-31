use std::collections::HashMap;
use anyhow::{anyhow, Result};
use reqwest::{Client, header, Response, Url};
use std::str::FromStr;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use mime::{Mime, APPLICATION_JSON};

#[derive(Parser, Debug)]
#[command(name = "HTTPie")]
#[command(author = "Pengsha Ying <yingfusheng@foxmail.com>")]
#[command(version = "1.0")]
#[command(about = "HTTPie is making APIs simple and intuitive for those building the tools of our time.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Get(Get),
    Post(Post),
}

#[derive(Args, Debug)]
struct Get {
    #[arg(value_parser = parse_url)]
    url: String,
}

#[derive(Args, Debug)]
struct Post {
    #[arg(value_parser = parse_url)]
    url: String,

    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KVPair>,
}

#[derive(Debug, Clone, PartialEq)]
struct KVPair {
    k: String,
    v: String,
}

impl FromStr for KVPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;

    Ok(s.into())
}

fn parse_kv_pair(s: &str) -> Result<KVPair> {
    Ok(s.parse()?)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let response = client.get(&args.url).send().await?;
    Ok(print_resp(response).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    };
    let response = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(response).await?)
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_header(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    print!("\n")
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => println!("{}", body)
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_header(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    let client = Client::builder().default_headers(headers).build()?;

    let result = match cli.command {
        Command::Get(ref args) => get(client, args).await,
        Command::Post(ref args) => post(client, args).await
    };

    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_works() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }

    #[test]
    fn parse_kv_pair_works() {
        assert!(parse_kv_pair("a").is_err());
        assert_eq!(
            parse_kv_pair("a=1").unwrap(),
            KVPair {
                k: "a".into(),
                v: "1".into(),
            }
        );

        assert_eq!(
            parse_kv_pair("b=").unwrap(),
            KVPair {
                k: "b".into(),
                v: "".into(),
            }
        )
    }
}
