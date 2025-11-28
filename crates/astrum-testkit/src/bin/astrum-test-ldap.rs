use astrum_testkit::ldap::{LdapTestServer, fixture};

fn port(position: usize) -> u16 {
    std::env::args()
        .nth(position)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let server = LdapTestServer::start_on(fixture(), port(1), port(3)).await;
    if let Some(path) = std::env::args().nth(2).filter(|path| !path.is_empty()) {
        std::fs::write(&path, &server.certificates.ca_pem).expect("the CA file can be written");
    }
    println!(
        "{{\"port\":{},\"ldapsPort\":{}}}",
        server.port, server.ldaps_port
    );
    let _ = tokio::signal::ctrl_c().await;
    server.stop();
}
