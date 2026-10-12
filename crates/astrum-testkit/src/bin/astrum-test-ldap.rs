use astrum_testkit::ldap::{LdapTestServer, fixture};

#[tokio::main]
async fn main() {
    let port = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let server = LdapTestServer::start_on(fixture(), port, 0).await;
    if let Some(path) = std::env::args().nth(2) {
        std::fs::write(&path, &server.certificates.ca_pem).expect("the CA file can be written");
    }
    println!(
        "{{\"port\":{},\"ldapsPort\":{}}}",
        server.port, server.ldaps_port
    );
    let _ = tokio::signal::ctrl_c().await;
    server.stop();
}
