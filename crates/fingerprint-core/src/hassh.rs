//! HASSH SSH fingerprintimplement
//!
//! HASSH is Salesforce opensend SSH client/serverfingerprintidentifymethod.
//! similar于 JA3 for TLS, HASSH for identify SSH client and server.
//!
//! ## reference
//! - paper: "HASSH - Profiling Method for SSH Clients and Servers" (Salesforce, 2018)
//! - GitHub: https://github.com/salesforce/hassh
//!
//! ## algorithm
//! HASSH = MD5(Client KEX Algorithms;Encryption Algorithms;MAC Algorithms;Compression Algorithms)

use serde::{Deserialize, Serialize};

/// HASSH SSH clientfingerprint
///
/// format: MD5(KEX;EncryptionAlgs;MACAlgs;CompressionAlgs)
///
/// ## Examples
/// ```
/// use fingerprint_core::hassh::HASSH;
///
/// let hassh = HASSH::generate(
/// &["diffie-hellman-group14-sha1", "diffie-hellman-group-exchange-sha256"],
/// &["aes128-ctr", "aes256-ctr"],
/// &["hmac-sha2-256", "hmac-sha2-512"],
/// &["none", "zlib@openssh.com"],
/// );
/// assert!(!hassh.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HASSH {
    /// keyswapalgorithmlist (semicolon-separated)
    pub kex_algorithms: String,

    /// encryptionalgorithmlist (semicolon-separated)
    pub encryption_algorithms: String,

    /// MAC algorithmlist (semicolon-separated)
    pub mac_algorithms: String,

    /// compressionalgorithmlist (semicolon-separated)
    pub compression_algorithms: String,

    /// complete HASSH string ( for Calculatehash)
    pub hassh_string: String,

    /// HASSH fingerprint (MD5 hash)
    pub fingerprint: String,

    /// SSH clienttype (infer)
    pub client_type: Option<String>,
}

impl HASSH {
    /// Generate HASSH fingerprint
    ///
    /// # Parameters
    /// - `kex_algorithms`: keyswapalgorithmlist
    /// - `encryption_algorithms`: encryptionalgorithmlist
    /// - `mac_algorithms`: MAC algorithmlist
    /// - `compression_algorithms`: compressionalgorithmlist
    ///
    /// # Returns
    /// HASSH fingerprintstruct
    pub fn generate(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
        mac_algorithms: &[&str],
        compression_algorithms: &[&str],
    ) -> Self {
        // connectionalgorithmlist (usesemicolon-separated)
        let kex_str = kex_algorithms.join(";");
        let enc_str = encryption_algorithms.join(";");
        let mac_str = mac_algorithms.join(";");
        let comp_str = compression_algorithms.join(";");

        // Build HASSH string
        let hassh_string = format!("{};{};{};{}", kex_str, enc_str, mac_str, comp_str);

        // Calculate MD5 hash
        let fingerprint = Self::md5_hash(&hassh_string);

        // inferclienttype
        let client_type = Self::infer_client_type(kex_algorithms, encryption_algorithms);

        Self {
            kex_algorithms: kex_str,
            encryption_algorithms: enc_str,
            mac_algorithms: mac_str,
            compression_algorithms: comp_str,
            hassh_string,
            fingerprint,
            client_type,
        }
    }

    /// Calculate MD5 hash
    fn md5_hash(input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    /// infer SSH clienttype
    fn infer_client_type(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
    ) -> Option<String> {
        // based onalgorithmtraitinferclienttype

        // OpenSSH trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("curve25519-sha256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("chacha20-poly1305"))
        {
            return Some("OpenSSH".to_string());
        }

        // PuTTY trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("ecdh-sha2-nistp256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("aes256-ctr"))
        {
            return Some("PuTTY".to_string());
        }

        // Paramiko (Python) trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("diffie-hellman-group14-sha1"))
            && !kex_algorithms.iter().any(|&k| k.contains("curve25519"))
        {
            return Some("Paramiko".to_string());
        }

        // libssh trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("ecdh-sha2-nistp521"))
        {
            return Some("libssh".to_string());
        }

        None
    }

    /// from SSH KEX_INIT messageParse HASSH
    ///
    /// SSH KEX_INIT messageincludingalgorithmnegotiateinfo
    pub fn from_kex_init(kex_init: &SSHKexInit) -> Self {
        Self::generate(
            &kex_init
                .kex_algorithms
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .encryption_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .mac_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .compression_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
    }
}

impl std::fmt::Display for HASSH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint)
    }
}

/// HASSH Server - SSH serverfingerprint
///
/// format: MD5(Server KEX;EncryptionAlgs;MACAlgs;CompressionAlgs)
///
/// ## Examples
/// ```
/// use fingerprint_core::hassh::HASSHServer;
///
/// let hassh_server = HASSHServer::generate(
/// &["diffie-hellman-group14-sha256"],
/// &["aes256-ctr"],
/// &["hmac-sha2-512"],
/// &["none"],
/// );
/// assert!(!hassh_server.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HASSHServer {
    /// keyswapalgorithmlist (semicolon-separated)
    pub kex_algorithms: String,

    /// encryptionalgorithmlist (semicolon-separated)
    pub encryption_algorithms: String,

    /// MAC algorithmlist (semicolon-separated)
    pub mac_algorithms: String,

    /// compressionalgorithmlist (semicolon-separated)
    pub compression_algorithms: String,

    /// complete HASSH Server string
    pub hassh_server_string: String,

    /// HASSH Server fingerprint (MD5 hash)
    pub fingerprint: String,

    /// SSH servertype (infer)
    pub server_type: Option<String>,
}

impl HASSHServer {
    /// Generate HASSH Server fingerprint
    pub fn generate(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
        mac_algorithms: &[&str],
        compression_algorithms: &[&str],
    ) -> Self {
        let kex_str = kex_algorithms.join(";");
        let enc_str = encryption_algorithms.join(";");
        let mac_str = mac_algorithms.join(";");
        let comp_str = compression_algorithms.join(";");

        let hassh_server_string = format!("{};{};{};{}", kex_str, enc_str, mac_str, comp_str);

        let fingerprint = Self::md5_hash(&hassh_server_string);

        let server_type = Self::infer_server_type(kex_algorithms, encryption_algorithms);

        Self {
            kex_algorithms: kex_str,
            encryption_algorithms: enc_str,
            mac_algorithms: mac_str,
            compression_algorithms: comp_str,
            hassh_server_string,
            fingerprint,
            server_type,
        }
    }

    /// Calculate MD5 hash
    fn md5_hash(input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }

    /// infer SSH servertype
    fn infer_server_type(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
    ) -> Option<String> {
        // OpenSSH Server
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("curve25519-sha256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("chacha20-poly1305"))
        {
            return Some("OpenSSH".to_string());
        }

        // Dropbear
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("diffie-hellman-group1-sha1"))
            && encryption_algorithms.len() < 5
        {
            return Some("Dropbear".to_string());
        }

        // libssh
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("ecdh-sha2-nistp521"))
        {
            return Some("libssh".to_string());
        }

        None
    }

    /// from SSH KEX_INIT messageParse HASSH Server
    pub fn from_kex_init(kex_init: &SSHKexInit) -> Self {
        Self::generate(
            &kex_init
                .kex_algorithms
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .encryption_algorithms_server_to_client
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .mac_algorithms_server_to_client
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .compression_algorithms_server_to_client
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
    }
}

impl std::fmt::Display for HASSHServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint)
    }
}

/// SSH KEX_INIT messagestruct
///
/// including SSH keyswapInitializemessageallalgorithmlist
#[derive(Debug, Clone, Default)]
pub struct SSHKexInit {
    /// keyswapalgorithm
    pub kex_algorithms: Vec<String>,

    /// serverhostkeyalgorithm
    pub server_host_key_algorithms: Vec<String>,

    /// client to serverencryptionalgorithm
    pub encryption_algorithms_client_to_server: Vec<String>,

    /// server to clientencryptionalgorithm
    pub encryption_algorithms_server_to_client: Vec<String>,

    /// client to server MAC algorithm
    pub mac_algorithms_client_to_server: Vec<String>,

    /// server to client MAC algorithm
    pub mac_algorithms_server_to_client: Vec<String>,

    /// client to servercompressionalgorithm
    pub compression_algorithms_client_to_server: Vec<String>,

    /// server to clientcompressionalgorithm
    pub compression_algorithms_server_to_client: Vec<String>,
}

impl SSHKexInit {
    /// Create a new KEX_INIT message
    pub fn new() -> Self {
        Self::default()
    }

    /// from originalbeginning SSH countpacketParse (simplified version)
    ///
    /// Note: this isanSimplified implementation, complete SSH protocolParseneedmorecomplexstatus机
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        // SSH protocolformatcomplex, hereprovidebasicframework
        // actualapplication in shouldusespecifically SSH protocolParselibrary

        if data.len() < 16 {
            return Err("countpackettoo short".to_string());
        }

        // SSH KEX_INIT messagetype as 20 (SSH_MSG_KEXINIT)
        if data[0] != 20 {
            return Err("is not KEX_INIT message".to_string());
        }

        // hereshouldParse name-list field
        // due to SSH protocolParsecomplex, temporary when returnemptystruct
        Ok(Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hassh_generation() {
        let hassh = HASSH::generate(
            &[
                "diffie-hellman-group14-sha1",
                "diffie-hellman-group-exchange-sha256",
            ],
            &["aes128-ctr", "aes192-ctr", "aes256-ctr"],
            &["hmac-sha2-256", "hmac-sha2-512"],
            &["none", "zlib@openssh.com"],
        );

        assert!(!hassh.fingerprint.is_empty());
        assert_eq!(hassh.fingerprint.len(), 32); // MD5 hashlength
        assert!(hassh.kex_algorithms.contains("diffie-hellman"));
        assert!(hassh.encryption_algorithms.contains("aes128-ctr"));
    }

    #[test]
    fn test_hassh_openssh_detection() {
        let hassh = HASSH::generate(
            &["curve25519-sha256", "ecdh-sha2-nistp256"],
            &["chacha20-poly1305@openssh.com", "aes256-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(hassh.client_type, Some("OpenSSH".to_string()));
    }

    #[test]
    fn test_hassh_putty_detection() {
        let hassh = HASSH::generate(
            &["ecdh-sha2-nistp256", "diffie-hellman-group14-sha1"],
            &["aes256-ctr", "aes192-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(hassh.client_type, Some("PuTTY".to_string()));
    }

    #[test]
    fn test_hassh_display() {
        let hassh = HASSH::generate(&["test"], &["test"], &["test"], &["none"]);
        let displayed = format!("{}", hassh);
        assert_eq!(displayed, hassh.fingerprint);
    }

    #[test]
    fn test_hassh_server_generation() {
        let hassh_server = HASSHServer::generate(
            &["diffie-hellman-group14-sha256"],
            &["aes256-ctr", "aes128-ctr"],
            &["hmac-sha2-512", "hmac-sha2-256"],
            &["none"],
        );

        assert!(!hassh_server.fingerprint.is_empty());
        assert_eq!(hassh_server.fingerprint.len(), 32);
    }

    #[test]
    fn test_hassh_server_openssh_detection() {
        let hassh_server = HASSHServer::generate(
            &["curve25519-sha256@libssh.org"],
            &["chacha20-poly1305@openssh.com"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(hassh_server.server_type, Some("OpenSSH".to_string()));
    }

    #[test]
    fn test_hassh_empty_algorithms() {
        let hassh = HASSH::generate(&[], &[], &[], &[]);
        assert!(!hassh.fingerprint.is_empty());
        assert_eq!(hassh.hassh_string, ";;;");
    }

    #[test]
    fn test_ssh_kex_init_creation() {
        let kex_init = SSHKexInit::new();
        assert!(kex_init.kex_algorithms.is_empty());
    }

    #[test]
    fn test_ssh_kex_init_parse_invalid() {
        let data = vec![0u8; 10];
        let result = SSHKexInit::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_hassh_string_format() {
        let hassh = HASSH::generate(&["kex1", "kex2"], &["enc1"], &["mac1"], &["comp1"]);
        assert_eq!(hassh.hassh_string, "kex1;kex2;enc1;mac1;comp1");
    }
}

/// JA4SSH - SSH fingerprint (JA4 style)
///
/// similar于 HASSH, butuse SHA256 rather than MD5, 并adopt JA4 seriesformatstyle
///
/// format: c{kex_count:02}{enc_count:02}{mac_count:02}_{kex_hash}_{enc_hash}_{mac_hash}
///
/// ## Examples
/// ```
/// use fingerprint_core::hassh::JA4SSH;
///
/// let ja4ssh = JA4SSH::generate(
/// &["diffie-hellman-group14-sha256"],
/// &["aes256-ctr"],
/// &["hmac-sha2-256"],
/// &["none"],
/// );
/// assert!(!ja4ssh.fingerprint_string().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA4SSH {
    /// 'c' for client, 's' for server
    pub direction: char,

    /// keyswapalgorithmcount
    pub kex_count: usize,

    /// encryptionalgorithmcount
    pub encryption_count: usize,

    /// MAC algorithmcount
    pub mac_count: usize,

    /// compressionalgorithmcount
    pub compression_count: usize,

    /// KEX algorithmhash (SHA256 front 6-bit)
    pub kex_hash: String,

    /// encryptionalgorithmhash (SHA256 front 6-bit)
    pub encryption_hash: String,

    /// MAC algorithmhash (SHA256 front 6-bit)
    pub mac_hash: String,

    /// compressionalgorithmhash (SHA256 front 6-bit)
    pub compression_hash: String,

    /// clienttype (infer)
    pub client_type: Option<String>,
}

impl JA4SSH {
    /// Generate JA4SSH clientfingerprint
    ///
    /// # Parameters
    /// - `kex_algorithms`: keyswapalgorithmlist
    /// - `encryption_algorithms`: encryptionalgorithmlist
    /// - `mac_algorithms`: MAC algorithmlist
    /// - `compression_algorithms`: compressionalgorithmlist
    pub fn generate(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
        mac_algorithms: &[&str],
        compression_algorithms: &[&str],
    ) -> Self {
        // Calculateeachalgorithmlist SHA256 hash
        let kex_hash = Self::compute_hash(kex_algorithms);
        let enc_hash = Self::compute_hash(encryption_algorithms);
        let mac_hash = Self::compute_hash(mac_algorithms);
        let comp_hash = Self::compute_hash(compression_algorithms);

        // inferclienttype
        let client_type = Self::infer_client_type(kex_algorithms, encryption_algorithms);

        Self {
            direction: 'c', // client
            kex_count: kex_algorithms.len().min(99),
            encryption_count: encryption_algorithms.len().min(99),
            mac_count: mac_algorithms.len().min(99),
            compression_count: compression_algorithms.len().min(99),
            kex_hash,
            encryption_hash: enc_hash,
            mac_hash,
            compression_hash: comp_hash,
            client_type,
        }
    }

    /// Generate JA4SSH serverfingerprint
    pub fn generate_server(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
        mac_algorithms: &[&str],
        compression_algorithms: &[&str],
    ) -> Self {
        let mut ssh = Self::generate(
            kex_algorithms,
            encryption_algorithms,
            mac_algorithms,
            compression_algorithms,
        );
        ssh.direction = 's'; // server
        ssh.client_type = Self::infer_server_type(kex_algorithms, encryption_algorithms);
        ssh
    }

    /// Calculatealgorithmlist SHA256 hash (getfront 6-bit)
    fn compute_hash(algorithms: &[&str]) -> String {
        use sha2::{Digest, Sha256};

        if algorithms.is_empty() {
            return "000000".to_string();
        }

        let combined = algorithms.join(";");
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();
        let hex = format!("{:x}", result);
        hex[0..6].to_string()
    }

    /// infer SSH clienttype
    fn infer_client_type(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
    ) -> Option<String> {
        // OpenSSH trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("curve25519-sha256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("chacha20-poly1305"))
        {
            return Some("OpenSSH".to_string());
        }

        // PuTTY trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("ecdh-sha2-nistp256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("aes256-ctr"))
        {
            return Some("PuTTY".to_string());
        }

        // Paramiko (Python) trait
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("diffie-hellman-group14-sha1"))
            && !kex_algorithms.iter().any(|&k| k.contains("curve25519"))
        {
            return Some("Paramiko".to_string());
        }

        None
    }

    /// infer SSH servertype
    fn infer_server_type(
        kex_algorithms: &[&str],
        encryption_algorithms: &[&str],
    ) -> Option<String> {
        // OpenSSH Server
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("curve25519-sha256"))
            && encryption_algorithms
                .iter()
                .any(|&e| e.contains("chacha20-poly1305"))
        {
            return Some("OpenSSH".to_string());
        }

        // Dropbear
        if kex_algorithms
            .iter()
            .any(|&k| k.contains("diffie-hellman-group1-sha1"))
            && encryption_algorithms.len() < 5
        {
            return Some("Dropbear".to_string());
        }

        None
    }

    /// convert tostandard JA4SSH fingerprintstring
    /// format: c{kex:02}{enc:02}{mac:02}{comp:02}_{kex_hash}_{enc_hash}_{mac_hash}_{comp_hash}
    pub fn fingerprint_string(&self) -> String {
        format!(
            "{}{:02}{:02}{:02}{:02}_{}_{}_{}_{}",
            self.direction,
            self.kex_count,
            self.encryption_count,
            self.mac_count,
            self.compression_count,
            self.kex_hash,
            self.encryption_hash,
            self.mac_hash,
            self.compression_hash
        )
    }

    /// from SSH KEX_INIT messageGenerate JA4SSH
    pub fn from_kex_init(kex_init: &SSHKexInit) -> Self {
        Self::generate(
            &kex_init
                .kex_algorithms
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .encryption_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .mac_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &kex_init
                .compression_algorithms_client_to_server
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        )
    }
}

impl std::fmt::Display for JA4SSH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fingerprint_string())
    }
}

#[cfg(test)]
mod ja4ssh_tests {
    use super::*;

    #[test]
    fn test_ja4ssh_generation() {
        let ja4ssh = JA4SSH::generate(
            &["diffie-hellman-group14-sha256", "ecdh-sha2-nistp256"],
            &["aes256-ctr", "aes128-ctr"],
            &["hmac-sha2-256", "hmac-sha2-512"],
            &["none", "zlib@openssh.com"],
        );

        assert_eq!(ja4ssh.direction, 'c');
        assert_eq!(ja4ssh.kex_count, 2);
        assert_eq!(ja4ssh.encryption_count, 2);
        assert_eq!(ja4ssh.mac_count, 2);
        assert_eq!(ja4ssh.compression_count, 2);
        assert_eq!(ja4ssh.kex_hash.len(), 6);
        assert_eq!(ja4ssh.encryption_hash.len(), 6);
    }

    #[test]
    fn test_ja4ssh_server() {
        let ja4ssh = JA4SSH::generate_server(
            &["diffie-hellman-group14-sha256"],
            &["aes256-ctr"],
            &["hmac-sha2-512"],
            &["none"],
        );

        assert_eq!(ja4ssh.direction, 's');
        assert_eq!(ja4ssh.kex_count, 1);
    }

    #[test]
    fn test_ja4ssh_openssh_detection() {
        let ja4ssh = JA4SSH::generate(
            &["curve25519-sha256", "ecdh-sha2-nistp256"],
            &["chacha20-poly1305@openssh.com", "aes256-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(ja4ssh.client_type, Some("OpenSSH".to_string()));
    }

    #[test]
    fn test_ja4ssh_putty_detection() {
        let ja4ssh = JA4SSH::generate(
            &["ecdh-sha2-nistp256", "diffie-hellman-group14-sha1"],
            &["aes256-ctr", "aes128-ctr"],
            &["hmac-sha2-256"],
            &["none"],
        );

        assert_eq!(ja4ssh.client_type, Some("PuTTY".to_string()));
    }

    #[test]
    fn test_ja4ssh_fingerprint_string() {
        let ja4ssh = JA4SSH::generate(&["test-kex"], &["test-enc"], &["test-mac"], &["none"]);

        let fp = ja4ssh.fingerprint_string();
        assert!(fp.starts_with("c01010101_"));
        assert!(fp.contains('_'));
    }

    #[test]
    fn test_ja4ssh_display() {
        let ja4ssh = JA4SSH::generate(&["kex"], &["enc"], &["mac"], &["none"]);

        let displayed = format!("{}", ja4ssh);
        assert_eq!(displayed, ja4ssh.fingerprint_string());
    }

    #[test]
    fn test_ja4ssh_empty_algorithms() {
        let ja4ssh = JA4SSH::generate(&[], &[], &[], &[]);

        assert_eq!(ja4ssh.kex_count, 0);
        assert_eq!(ja4ssh.kex_hash, "000000");
        assert!(ja4ssh.fingerprint_string().contains("c00000000_"));
    }

    #[test]
    fn test_ja4ssh_hash_consistency() {
        // same样algorithmshouldproducesameofhash
        let ja4ssh1 = JA4SSH::generate(&["algo1", "algo2"], &["enc"], &["mac"], &["none"]);
        let ja4ssh2 = JA4SSH::generate(&["algo1", "algo2"], &["enc"], &["mac"], &["none"]);

        assert_eq!(ja4ssh1.kex_hash, ja4ssh2.kex_hash);
        assert_eq!(ja4ssh1.encryption_hash, ja4ssh2.encryption_hash);
    }
}
