// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, BufReader};
use std::path::{PathBuf};

use chrono::{DateTime, Utc};
use uuid::Uuid;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::cipher::{CipherSuite, CipherString};


const AUTH_URL: &str = "https://identity.bitwarden.com/connect/token";
const BASE_URL: &str = "https://api.bitwarden.com";


#[derive(Debug, failure::Fail)]
pub enum ApiError {
        #[fail(display = "prelogin failed: {}", error)]
        PreloginFailed {
                error: String,
        },
        #[fail(display = "authentication failed: {}", error)]
        LoginFailed {
                error: String,
        },
        #[fail(display = "failed to retrieve {}: {}", endpoint, error)]
        RequestFailed {
                endpoint: String,
                error: String,
        },
        #[fail(display = "failed to write sync data: {}", error)]
        VaultDataWriteFailed {
                error: String,
        },
        #[fail(display = "failed to read sync data: {}", error)]
        VaultDataReadFailed {
                error: String,
        },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthData {
        access_token: String,
        expires_in: usize,
        token_type: String,

        kdf: usize,
        pub kdf_iterations: usize,

        #[serde(skip)]
        pub cipher: CipherSuite,
}


#[derive(Debug, Deserialize)]
struct PreloginResponseData {
        #[serde(alias = "Kdf")]
        kdf: usize,
        #[serde(alias = "KdfIterations")]
        kdf_iterations: usize,
}


#[derive(Debug, Deserialize)]
struct LoginResponseData {
        access_token: String,
        expires_in: usize,
        token_type: String,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
        #[serde(alias = "Object")]
        object: String,
        #[serde(alias = "Id")]
        pub uuid: Uuid,
        #[serde(alias = "Name")]
        pub name: String,
        #[serde(alias = "Email")]
        pub email: String,
        #[serde(alias = "EmailVerified")]
        pub email_verified: bool,
        #[serde(alias = "Premium")]
        pub premium: bool,
        #[serde(alias = "MasterPasswordHint")]
        pub master_password_hint: Option<String>,
        #[serde(alias = "Culture")]
        pub language: String,
        #[serde(alias = "TwoFactorEnabled")]
        pub tfa_enabled: bool,
        #[serde(alias = "Key")]
        pub key: CipherString,
        #[serde(alias = "PrivateKey")]
        pub private_key: CipherString,
        #[serde(alias = "SecurityStamp")]
        pub security_stamp: String,
        #[serde(alias = "Organizations")]
        pub organizations: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Folder {
        #[serde(alias = "Object")]
        object: String,
        #[serde(alias = "Id")]
        pub uuid: Uuid,
        #[serde(alias = "Name")]
        pub name: CipherString,
        #[serde(alias = "RevisionDate")]
        pub last_changed: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CipherEntryFields {
        #[serde(alias = "Type")]
        pub type_: usize,
        #[serde(alias = "Name")]
        pub name: CipherString,
        #[serde(alias = "Value")]
        pub value: CipherString,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CipherEntryHistory {
        #[serde(alias = "Password")]
        pub password: String,
        #[serde(alias = "LastUsedDate")]
        pub last_used_date: DateTime<Utc>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CipherEntryUriMatch {
        #[serde(alias  = "Uri")]
        pub uri: CipherString,
        #[serde(alias  = "Match")]
        pub match_: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CipherEntryData {
        #[serde(alias = "Uri")]
        pub uri: Option<CipherString>,
        #[serde(alias = "Uris")]
        pub uris: Option<Vec<CipherEntryUriMatch>>,
        #[serde(alias = "Username")]
        pub username: CipherString,
        #[serde(alias = "Password")]
        pub password: CipherString,
        #[serde(alias = "PasswordRevisionDate")]
        pub assword_last_changed: Option<DateTime<Utc>>,
        #[serde(alias = "Totp")]
        pub totp: Option<String>,
        #[serde(alias = "Name")]
        pub name: CipherString,
        #[serde(alias = "Notes")]
        pub notes: Option<String>,
        #[serde(alias = "Fields")]
        pub fields: Option<Vec<CipherEntryFields>>,
        #[serde(alias = "PasswordHistory")]
        pub password_history: Option<Vec<CipherEntryHistory>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CipherEntry {
        #[serde(alias = "Object")]
        object: String,
        #[serde(alias = "CollectionIds")]
        pub collection_ids: Vec<Uuid>,
        #[serde(alias = "FolderId")]
        pub folder_id: Option<Uuid>,
        #[serde(alias = "Favorite")]
        pub favorite: bool,
        #[serde(alias = "Edit")]
        pub edit: bool,
        #[serde(alias = "Id")]
        pub uuid: Uuid,
        #[serde(alias = "OrganizationId")]
        pub organization_id: Option<Uuid>,
        #[serde(alias = "Type")]
        pub type_: usize,
        #[serde(alias = "Data")]
        pub data: CipherEntryData,
        #[serde(alias = "Name")]
        pub name: CipherString,
        #[serde(alias = "Notes")]
        pub notes: Option<String>,
        #[serde(alias = "Login", skip)]
        pub login: Option<CipherEntryData>,
        #[serde(alias = "Card")]
        pub card: Option<String>,
        #[serde(alias = "Identity")]
        pub identity: Option<String>,
        #[serde(alias = "SecureNote")]
        pub secure_note: Option<String>,
        #[serde(alias = "Fields")]
        pub fields: Option<Vec<CipherEntryFields>>,
        #[serde(alias = "PasswordHistory")]
        pub password_history: Option<Vec<CipherEntryHistory>>,
        #[serde(alias = "Attachments")]
        pub attachments: Option<String>,
        #[serde(alias = "OrganizationUseTotp")]
        pub organization_tfa: bool,
        #[serde(alias = "RevisionDate")]
        pub last_changed: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Domains {
        // TODO
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VaultData {
        #[serde(alias = "Object")]
        object: String,
        #[serde(alias = "Profile")]
        pub profile: Profile,
        #[serde(alias = "Folders")]
        pub folders: Vec<Folder>,
        #[serde(alias = "Collections")]
        pub collections: Vec<String>,
        #[serde(alias = "Ciphers")]
        pub ciphers: Vec<CipherEntry>,
        #[serde(alias = "Domains", skip)]
        domains: Option<Domains>,
}

#[derive(Debug)]
pub struct AppData {
        pub auth: AuthData,
        pub vault: VaultData,
}


fn perform_prelogin(client: &reqwest::Client, email: &str) -> Result<PreloginResponseData, ApiError> {
        let url = format!("{}/accounts/prelogin", BASE_URL);

        let mut data = HashMap::new();
        data.insert("email", email);

        let mut response = client.post(&url)
                .json(&data)
                .send()
                .map_err(|e| ApiError::PreloginFailed { error: e.to_string() })?;

        if response.status().is_success() {
                let data: PreloginResponseData = response
                        .json()
                        .map_err(|e| ApiError::PreloginFailed { error: e.to_string() })?;

                Ok(data)
        } else {
                Err(ApiError::PreloginFailed { error: format!("{:?}", response.status()) })
        }
}


fn perform_token_auth(client: &reqwest::Client, email: &str, cipher: &CipherSuite)
        -> Result<LoginResponseData, ApiError>
{
        let device_id = Uuid::new_v4().to_hyphenated().to_string();

        let mut data = HashMap::new();
        data.insert("grant_type", "password");
        data.insert("username", email);
        data.insert("scope", "api offline_access");
        data.insert("client_id", "connector");
        data.insert("deviceType", "3");
        data.insert("deviceIdentifier", &device_id);
        data.insert("deviceName", "bwtui");
        data.insert("password", &cipher.master_key_hash);

        let mut response = client.post(AUTH_URL)
                .form(&data)
                .send()
                .map_err(|e| ApiError::LoginFailed { error: e.to_string() })?;

        if response.status().is_success() {
                let data: LoginResponseData = response
                        .json()
                        .map_err(|e| ApiError::LoginFailed { error: e.to_string() })?;

                Ok(data)
        } else {
                Err(ApiError::LoginFailed { error: format!("{:?}", response.status()) })
        }
}


pub fn authenticate(email: &str, password: &str) -> Result<AuthData, ApiError> {
        let client = reqwest::Client::new();

        let PreloginResponseData { kdf, kdf_iterations } =
                perform_prelogin(&client, email)?;

        let cipher = CipherSuite::from(email, password, kdf_iterations);

        let LoginResponseData { access_token, expires_in, token_type } =
                perform_token_auth(&client, email, &cipher)?;

        Ok(AuthData {
                access_token,
                expires_in,
                token_type,
                kdf,
                kdf_iterations,
                cipher,
        })
}


pub fn sync(auth_data: &AuthData) -> Result<VaultData, ApiError> {
        let url = format!("{}/sync", BASE_URL);

        let map_reqwest_err = |e: reqwest::Error| {
                ApiError::RequestFailed { endpoint: url.clone(), error: e.to_string() }
        };


        let mut headers = HeaderMap::new();
        let auth_header = format!("{} {}", auth_data.token_type, auth_data.access_token);
        headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&auth_header).unwrap());

        let client = reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .map_err(map_reqwest_err)?;

        let mut response = client.get(&url)
                .send()
                .map_err(map_reqwest_err)?;

        if response.status().is_success() {
                let data: VaultData = response
                        .json()
                        .map_err(map_reqwest_err)?;

                Ok(data)
        } else {
                Err(ApiError::RequestFailed {
                        endpoint: url.clone(),
                        error: format!("{:?}", response.status())
                })
        }
}


fn get_app_data_path() -> Result<PathBuf, String> {
        let project_dirs = directories::ProjectDirs::from("", "", "bwtui")
                .ok_or("could not retrieve data directory path")?;

        let target_dir = project_dirs.data_local_dir();

        fs::create_dir_all(target_dir)
                .map_err(|_| "could not create data directory")?;

        let mut path = PathBuf::new();
        path.push(target_dir);

        Ok(path)
}


fn save_data_to<T>(filename: &str, data: &T) -> Result<(), ApiError>
        where T: Serialize
{
        let mut path = get_app_data_path()
                .map_err(|error| ApiError::VaultDataWriteFailed { error })?;
        path.push(filename);

        let file = File::create(path)
                .map_err(|e| ApiError::VaultDataWriteFailed { error: e.to_string() })?;

        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, data)
                .map_err(|e| ApiError::VaultDataWriteFailed { error: e.to_string() })
}


fn read_data_from<T>(filename: &str) -> Result<T, ApiError>
        where T: DeserializeOwned
{
        let mut path = get_app_data_path()
                .map_err(|error| ApiError::VaultDataReadFailed { error })?;
        path.push(filename);

        let file = File::open(path)
                .map_err(|e| ApiError::VaultDataReadFailed { error: e.to_string() })?;

        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
                .map_err(|e| ApiError::VaultDataReadFailed { error: e.to_string() })
}


pub fn read_app_data() -> Result<AppData, ApiError> {
        let auth = read_data_from("auth.json")?;
        let vault = read_data_from("vault.json")?;

        Ok(AppData {
                auth,
                vault,
        })
}


pub fn save_app_data(auth: &AuthData, vault: &VaultData) -> Result<(), ApiError> {
        save_data_to("auth.json", auth)?;
        save_data_to("vault.json", vault)?;

        Ok(())
}
