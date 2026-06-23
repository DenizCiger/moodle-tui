use crate::storage::{StorageError, config_dir};
use std::path::{Path, PathBuf};

pub fn trusted_certificate_file(base_url: &str) -> Result<PathBuf, StorageError> {
    let url = reqwest::Url::parse(base_url)
        .map_err(|error| StorageError::Message(format!("Invalid Moodle base URL: {error}")))?;
    trusted_certificate_file_in(&config_dir()?, &url)
}

fn trusted_certificate_file_in(
    config_dir: &Path,
    url: &reqwest::Url,
) -> Result<PathBuf, StorageError> {
    let host = url
        .host_str()
        .ok_or_else(|| StorageError::Message("Moodle base URL has no hostname".to_owned()))?;
    let safe_host: String = host
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect();
    let port_suffix = url
        .port()
        .map(|port| format!("-port-{port}"))
        .unwrap_or_default();
    Ok(config_dir
        .join("trusted-certificates")
        .join(format!("{safe_host}{port_suffix}.pem")))
}

pub fn load_trusted_certificates(
    base_url: &str,
) -> Result<Vec<reqwest::Certificate>, StorageError> {
    let path = trusted_certificate_file(base_url)?;
    load_trusted_certificates_from_path(&path)
}

fn load_trusted_certificates_from_path(
    path: &Path,
) -> Result<Vec<reqwest::Certificate>, StorageError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let pem = std::fs::read(&path).map_err(|error| {
        StorageError::Message(format!("Failed to read {}: {error}", path.display()))
    })?;
    let certificates = reqwest::Certificate::from_pem_bundle(&pem).map_err(|error| {
        StorageError::Message(format!(
            "Invalid certificate in {}: {error}",
            path.display()
        ))
    })?;
    if certificates.is_empty() {
        return Err(StorageError::Message(format!(
            "No certificates found in {}",
            path.display()
        )));
    }
    Ok(certificates)
}

#[cfg(test)]
mod tests {
    use super::{load_trusted_certificates_from_path, trusted_certificate_file_in};
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    const TEST_CERTIFICATE: &str = "-----BEGIN CERTIFICATE-----\n\
MIIBtjCCAVugAwIBAgITBmyf1XSXNmY/Owua2eiedgPySjAKBggqhkjOPQQDAjA5\n\
MQswCQYDVQQGEwJVUzEPMA0GA1UEChMGQW1hem9uMRkwFwYDVQQDExBBbWF6b24g\n\
Um9vdCBDQSAzMB4XDTE1MDUyNjAwMDAwMFoXDTQwMDUyNjAwMDAwMFowOTELMAkG\n\
A1UEBhMCVVMxDzANBgNVBAoTBkFtYXpvbjEZMBcGA1UEAxMQQW1hem9uIFJvb3Qg\n\
Q0EgMzBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABCmXp8ZBf8ANm+gBG1bG8lKl\n\
ui2yEujSLtf6ycXYqm0fc4E7O5hrOXwzpcVOho6AF2hiRVd9RFgdszflZwjrZt6j\n\
QjBAMA8GA1UdEwEB/wQFMAMBAf8wDgYDVR0PAQH/BAQDAgGGMB0GA1UdDgQWBBSr\n\
ttvXBp43rDCGB5Fwx5zEGbF4wDAKBggqhkjOPQQDAgNJADBGAiEA4IWSoxe3jfkr\n\
BqWTrBqYaGFy+uGh0PsceGCmQ5nFuMQCIQCcAu/xlJyzlvnrxir4tiz+OpAUFteM\n\
YyRIHN8wfdVoOw==\n\
-----END CERTIFICATE-----\n";

    #[test]
    fn scopes_certificate_to_moodle_hostname() {
        let root = tempdir().unwrap();
        let url = reqwest::Url::parse("https://moodle.example.edu/moodle/").unwrap();
        let path = trusted_certificate_file_in(root.path(), &url).unwrap();
        assert!(path.ends_with("trusted-certificates/moodle.example.edu.pem"));
    }

    #[test]
    fn uses_port_and_portable_ipv6_filename() {
        let root = tempdir().unwrap();
        let url = reqwest::Url::parse("https://[2001:db8::1]:8443/moodle/").unwrap();
        let path = trusted_certificate_file_in(root.path(), &url).unwrap();
        assert!(path.ends_with("trusted-certificates/_2001_db8__1_-port-8443.pem"));
    }

    #[test]
    fn absent_certificate_returns_empty_collection() {
        let root = tempdir().unwrap();
        let path = certificate_path(root.path(), "https://moodle.example.edu/");
        let result = load_trusted_certificates_from_path(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn rejects_malformed_certificate() {
        let root = tempdir().unwrap();
        write_certificate(
            root.path(),
            "https://moodle.example.edu/",
            b"not a certificate",
        );
        let path = certificate_path(root.path(), "https://moodle.example.edu/");
        assert!(load_trusted_certificates_from_path(&path).is_err());
    }

    #[test]
    fn loads_every_certificate_in_bundle() {
        let root = tempdir().unwrap();
        let bundle = format!("{TEST_CERTIFICATE}{TEST_CERTIFICATE}");
        write_certificate(
            root.path(),
            "https://moodle.example.edu/",
            bundle.as_bytes(),
        );
        let path = certificate_path(root.path(), "https://moodle.example.edu/");
        let certificates = load_trusted_certificates_from_path(&path).unwrap();
        assert_eq!(certificates.len(), 2);
    }

    fn certificate_path(root: &Path, base_url: &str) -> PathBuf {
        let url = reqwest::Url::parse(base_url).unwrap();
        trusted_certificate_file_in(root, &url).unwrap()
    }

    fn write_certificate(root: &Path, base_url: &str, contents: &[u8]) {
        let path = certificate_path(root, base_url);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}
