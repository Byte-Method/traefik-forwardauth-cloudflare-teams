use clap::Parser;
use std::{
    env,
    net::{AddrParseError, IpAddr, SocketAddr},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Cloudflare Teams domain
    #[arg(long, env = "CF_TEAMS_DOMAIN")]
    pub teams_domain: String,

    /// Server socket
    #[arg(short, long, env = "HOST", value_parser=parse_bind_addr, value_name = "ADDRESS", default_value = "[::1]:80")]
    pub bind: SocketAddr,
}

fn parse_bind_addr(s: &str) -> Result<SocketAddr, AddrParseError> {
    s.parse()
        .or_else(|_| -> Result<SocketAddr, AddrParseError> {
            let ip: IpAddr = s.parse()?;
            let port: u16 = env::var("PORT")
                .ok()
                .and_then(|port| port.parse().ok())
                .unwrap_or(80);

            Ok(SocketAddr::new(ip, port))
        })
}
