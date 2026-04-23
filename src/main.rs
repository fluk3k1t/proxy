use std::fs;

use rcgen::{
    CertificateParams, CertifiedKey, Issuer, KeyPair, SigningKey, generate_simple_self_signed,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_key_pair = KeyPair::generate().unwrap();

    let client_cert_param = CertificateParams::new(vec!["example.com".to_string()]).unwrap();

    let ca_pem_str = fs::read_to_string("ore_ca.cert").unwrap();
    let ca_signing_key = fs::read_to_string("ore_ca.key").unwrap();

    let ca_issuer =
        Issuer::from_ca_cert_pem(&ca_pem_str, KeyPair::from_pem(&ca_signing_key).unwrap()).unwrap();
    // println!("{:?}", ca_issuer);

    let cert = client_cert_param
        .signed_by(&client_key_pair, &ca_issuer)
        .unwrap();

    println!("{}", cert.pem());

    Ok(())
}
