
#[derive(Serialize, Deserialize, Debug)]
struct Profile {
    device_name: String,
    other_devices: Vec<String>,
}
