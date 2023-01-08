use crate::config::ConfigError;
use rustls::{Certificate, RootCertStore};
use rustls_pemfile::Item;
use std::{
    fs::{self, File},
    io::BufReader,
};

#[allow(unused_mut)]
pub fn load_certificates(mut files: Vec<String>) -> Result<RootCertStore, ConfigError> {
    #[cfg(target_os = "android")]
    {
        if let Ok(dir) = fs::read_dir("/system/etc/security/cacerts") {
            let sys_certs = dir.filter_map(|r| r.ok());
            for cert in sys_certs {
                if let Some(str) = cert.path().to_str() {
                    files.push(str.parse().unwrap());
                }
            }
        }
    }

    let mut certs = RootCertStore::empty();

    for file in &files {
        let mut file =
            BufReader::new(File::open(file).map_err(|err| ConfigError::Io(file.to_owned(), err))?);

        while let Ok(Some(item)) = rustls_pemfile::read_one(&mut file) {
            if let Item::X509Certificate(cert) = item {
                certs.add(&Certificate(cert))?;
            }
        }
    }

    if certs.is_empty() {
        for file in &files {
            certs.add(&Certificate(
                fs::read(file).map_err(|err| ConfigError::Io(file.to_owned(), err))?,
            ))?;
        }
    }

    for cert in rustls_native_certs::load_native_certs().map_err(ConfigError::NativeCertificate)? {
        certs.add(&Certificate(cert.0))?;
    }

    Ok(certs)
}

use rustls::client::{ServerCertVerified, ServerCertVerifier};

pub struct SkipVerify;

impl ServerCertVerifier for SkipVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item=&[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}
