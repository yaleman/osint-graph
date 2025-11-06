// use std::io::BufReader;

// use osint_graph_shared::error::OsintError;
// use rustls::pki_types::{CertificateDer, PrivateKeyDer};
// use tokio::fs::File;

// use crate::cli::CliOpts;

// // // Load public certificate from file.
// // pub async fn load_cert(cli: &CliOpts) -> Result<Vec<CertificateDer<'static>>, OsintError> {
// //     // Open certificate file.
// //     if let Some(cert_file) = cli.tls_cert.as_ref() {
// //         let certfile = File::open(cert_file).await?;
// //         let mut reader = BufReader::new(certfile.into_std().await);
// //         // Load and return certificate.
// //         rustls_pemfile::certs(&mut reader)
// //             .collect::<Result<Vec<_>, _>>()
// //             .map_err(|e| OsintError::Other(format!("Failed to load certificates: {}", e)))
// //     } else {
// //         Ok(vec![])
// //     }
// // }

// // Load private key from file.
// pub async fn load_private_key(cli: &CliOpts) -> Result<PrivateKeyDer<'static>, OsintError> {
//     // Open keyfile.
//     if let Some(key_file) = cli.tls_key.as_ref() {
//         let keyfile = File::open(key_file).await?;
//         let mut reader = BufReader::new(keyfile.into_std().await);

//         // Load and return a single private key.
//         match rustls_pemfile::private_key(&mut reader) {
//             Ok(Some(key)) => Ok(key),
//             Ok(_) => Err(OsintError::Other(
//                 "No private keys found in the key file".to_string(),
//             )),
//             Err(e) => Err(OsintError::Other(format!(
//                 "Failed to load private key: {}",
//                 e
//             ))),
//         }
//     } else {
//         Err(OsintError::Configuration(
//             "TLS key file not specified".to_string(),
//         ))
//     }
// }
