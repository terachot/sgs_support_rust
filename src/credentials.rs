//! เก็บข้อมูลเข้าสู่ระบบ SGS ใน Windows Credential Manager เท่านั้น

use anyhow::{Context, Result};
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};

const SERVICE: &str = "sgs_support_rust/sgs.bopp-obec.info";
const USER: &str = "production-login";

#[derive(Clone, Deserialize, Serialize)]
pub struct SavedCredentials {
    pub username: String,
    pub password: String,
}

fn entry() -> Result<Entry> {
    Entry::new(SERVICE, USER).context("เปิด Windows Credential Manager ไม่สำเร็จ")
}

pub fn load() -> Result<Option<SavedCredentials>> {
    match entry()?.get_password() {
        Ok(value) => Ok(Some(
            serde_json::from_str(&value).context("ข้อมูลเข้าสู่ระบบที่บันทึกไว้ไม่ถูกต้อง")?,
        )),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(error) => Err(error).context("อ่านรหัสผ่านที่บันทึกไว้ไม่สำเร็จ"),
    }
}

pub fn save(username: &str, password: &str) -> Result<()> {
    let value = serde_json::to_string(&SavedCredentials {
        username: username.to_owned(),
        password: password.to_owned(),
    })?;
    entry()?
        .set_password(&value)
        .context("บันทึกรหัสผ่านใน Windows Credential Manager ไม่สำเร็จ")
}

pub fn delete() -> Result<()> {
    match entry()?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(error) => Err(error).context("ลบรหัสผ่านที่บันทึกไว้ไม่สำเร็จ"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_credentials_round_trip() -> Result<()> {
        let value = SavedCredentials {
            username: "test-user".to_owned(),
            password: "example-only".to_owned(),
        };
        let serialized = serde_json::to_string(&value)?;
        let decoded: SavedCredentials = serde_json::from_str(&serialized)?;
        assert_eq!(decoded.username, value.username);
        assert_eq!(decoded.password, value.password);
        Ok(())
    }

    #[test]
    #[ignore = "เขียนและลบข้อมูลทดสอบใน Windows Credential Manager"]
    fn windows_credential_store_smoke() -> Result<()> {
        let test_user = format!("test-{}", std::process::id());
        let entry = Entry::new(SERVICE, &test_user)?;
        entry.set_password("example-only")?;
        let loaded = entry.get_password();
        let deleted = entry.delete_credential();
        assert_eq!(loaded?, "example-only");
        deleted?;
        assert!(matches!(entry.get_password(), Err(KeyringError::NoEntry)));
        Ok(())
    }
}
