#![cfg(feature = "backend-ipfs-api")]

use std::path::PathBuf;

use distrox_lib::profile::Profile;
use test_context::{test_context, AsyncTestContext};

async fn check_state_file_after_create(
    test_state_path: &PathBuf,
    key_name: &'static str,
    expect_latest_id: bool,
) -> Result<(), anyhow::Error> {
    let file = tokio::fs::read_to_string(test_state_path).await?;
    let toml = toml::from_str(&file)?;
    if let toml::Value::Table(ref tab) = toml {
        if let Some(toml::Value::String(s)) = tab.get("key_name") {
            assert_eq!(s, key_name);
        } else {
            panic!("Not available: 'key_name': {:?}", toml);
        }

        if let Some(toml::Value::String(_)) = tab.get("key_id") {
            // key_id available
        } else {
            panic!("Not available: 'key_id': {:?}", toml);
        }

        let has_latest_node = tab.get("latest_node").is_some();
        assert_eq!(has_latest_node, expect_latest_id);
    } else {
        panic!("Not a table: {:?}", toml);
    }

    Ok(())
}

struct ProfileStateContext {
    ipfs_host_addr: std::net::SocketAddr,
    test_state_dir: PathBuf,
    state_file_name: Option<String>,
}

impl ProfileStateContext {
    pub fn get_state_file_path(&mut self, filename: &'static str) -> PathBuf {
        self.state_file_name = Some(filename.to_string());
        self.test_state_dir.clone().join(filename)
    }
}

#[async_trait::async_trait]
impl AsyncTestContext for ProfileStateContext {
    async fn setup() -> Self {
        let ipfs_host_addr = std::env::var("IPFS_HOST_ADDR").unwrap().parse().unwrap();
        let test_state_dir = PathBuf::from(std::env::var("TEST_STATE_DIR").unwrap());
        ProfileStateContext {
            ipfs_host_addr,
            test_state_dir,
            state_file_name: None,
        }
    }

    async fn teardown(self) {
        let filepath = self
            .test_state_dir
            .join(self.state_file_name.unwrap_or_default());

        if filepath.exists() {
            tokio::fs::remove_file(filepath).await.unwrap();
        } else {
            eprintln!("Does not exist: {}", filepath.display());
        }
    }
}

#[test_context(ProfileStateContext)]
#[tokio::test]
async fn test_profile_create(ctx: &mut ProfileStateContext) {
    let test_state_path = ctx.get_state_file_path("test_profile_create.toml");

    let _profile = Profile::create(
        ctx.ipfs_host_addr,
        "test_profile_create".to_string(),
        test_state_path.clone(),
    )
    .await
    .unwrap();

    check_state_file_after_create(&test_state_path, "test_profile_create", false)
        .await
        .unwrap();
}
