#![cfg(feature = "backend-ipfs-api")]

use std::path::PathBuf;

mod images;

use distrox_lib::profile::Profile;

async fn check_state_file_after_create(
    test_state_path: &PathBuf,
    key_name: &str,
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

async fn get_state_cid(test_state_path: &PathBuf) -> Result<String, anyhow::Error> {
    let file = tokio::fs::read_to_string(test_state_path).await?;
    let toml = toml::from_str(&file)?;
    if let toml::Value::Table(ref tab) = toml {
        Ok({
            tab.get("latest_node")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
    } else {
        panic!("Not a table: {:?}", toml);
    }
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

async fn create_profile(
    ipfs_host_addr: &std::net::SocketAddr,
    tempdir: PathBuf,
    key_name: &'static str,
) -> Result<(PathBuf, Profile), anyhow::Error> {
    let test_state_path = tempdir.join(&format!("{}.toml", key_name));

    let profile = Profile::create(
        *ipfs_host_addr,
        key_name.to_string(),
        test_state_path.clone(),
    )
    .await
    .unwrap();

    check_state_file_after_create(&test_state_path, &key_name.to_string(), false)
        .await
        .unwrap();

    Ok((test_state_path, profile))
}

#[tokio::test]
async fn test_profile_post_text() {
    let _ = env_logger::try_init();
    let docker = testcontainers::clients::Cli::docker();
    let container = docker.run(crate::images::ipfs::Ipfs);
    let port = container.get_host_port_ipv4(5001);

    let ipfs_host_addr = format!("127.0.0.1:{}", port).parse().unwrap();
    let tempdir = tempdir::TempDir::new("test_profile_post_text").unwrap();

    let (test_state_path, mut profile) = create_profile(
        &ipfs_host_addr,
        tempdir.path().to_path_buf(),
        "test_profile_post_text",
    )
    .await
    .unwrap();

    let text = "testtext";
    profile.post_text(text.to_string()).await.unwrap();

    // After posting, the state isn't updated yet
    check_state_file_after_create(&test_state_path, "test_profile_post_text", false)
        .await
        .unwrap();

    // ... we need to save the profile for that.
    profile.safe().await.unwrap();
    check_state_file_after_create(&test_state_path, "test_profile_post_text", true)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_profile_post_two_texts() {
    let _ = env_logger::try_init();
    let docker = testcontainers::clients::Cli::docker();
    let container = docker.run(crate::images::ipfs::Ipfs);
    let port = container.get_host_port_ipv4(5001);

    let ipfs_host_addr = format!("127.0.0.1:{}", port).parse().unwrap();
    let tempdir = tempdir::TempDir::new("test_profile_post_text").unwrap();
    let (test_state_path, mut profile) = create_profile(
        &ipfs_host_addr,
        tempdir.path().to_path_buf(),
        "test_profile_post_two_texts",
    )
    .await
    .unwrap();

    {
        let text = "testtext1";
        profile.post_text(text.to_string()).await.unwrap();

        check_state_file_after_create(&test_state_path, "test_profile_post_two_texts", false)
            .await
            .unwrap();

        profile.safe().await.unwrap();
        check_state_file_after_create(&test_state_path, "test_profile_post_two_texts", true)
            .await
            .unwrap();
    }

    let first_cid = get_state_cid(&test_state_path).await.unwrap();

    {
        let text = "testtext2";
        profile.post_text(text.to_string()).await.unwrap();

        check_state_file_after_create(&test_state_path, "test_profile_post_two_texts", true)
            .await
            .unwrap();

        profile.safe().await.unwrap();
        check_state_file_after_create(&test_state_path, "test_profile_post_two_texts", true)
            .await
            .unwrap();
    }

    let second_cid = get_state_cid(&test_state_path).await.unwrap();

    assert_ne!(first_cid, second_cid);
}
