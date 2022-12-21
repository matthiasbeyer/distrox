#![cfg(feature = "backend-ipfs-api")]

use std::path::PathBuf;

mod images;

use distrox_lib::profile::Profile;

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

#[tokio::test]
async fn test_profile_create() {
    let _ = env_logger::try_init();
    let docker = testcontainers::clients::Cli::docker();
    let container = docker.run(crate::images::ipfs::Ipfs);
    let port = container.get_host_port_ipv4(5001);

    let ipfs_host_addr = format!("127.0.0.1:{}", port).parse().unwrap();

    let tempdir = tempdir::TempDir::new("test_profile_create").unwrap();
    let test_state_path = tempdir.path().join("test_profile_create.toml");

    let _profile = Profile::create(
        ipfs_host_addr,
        "test_profile_create".to_string(),
        test_state_path.clone(),
    )
    .await
    .unwrap();

    check_state_file_after_create(&test_state_path, "test_profile_create", false)
        .await
        .unwrap();
}
