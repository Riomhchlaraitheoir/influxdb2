use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let influx_url = "http://localhost:8888";
    let token = "some-token";

    let client = influxdb2::Client::new(influx_url, "org", token);

    println!("{:?}", client.labels()?);
    println!("{:?}", client.labels_by_org("some-org_id")?);
    println!("{:?}", client.find_label("some-label_id")?);
    let mut properties = HashMap::new();
    properties.insert("some-key".to_string(), "some-value".to_string());
    println!(
        "{:?}",
        client
            .create_label("some-org_id", "some-name", Some(properties))
            ?
    );
    println!(
        "{:?}",
        client
            .update_label(Some("some-name".to_string()), None, "some-label_id")
            ?
    );
    println!("{:?}", client.delete_label("some-label_id")?);
    Ok(())
}
