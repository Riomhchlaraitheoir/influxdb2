use chrono::Utc;
use influxdb2_derive::WriteDataPoint;

#[derive(Default, WriteDataPoint)]
#[measurement = "cpu_load_short"]
struct CpuLoadShort {
    #[influxdb(tag)]
    host: Option<String>,
    #[influxdb(tag)]
    region: Option<String>,
    #[influxdb(field)]
    value: f64,
    #[influxdb(timestamp)]
    time: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let org = "sahamee";
    let bucket = "bucket";
    let influx_url = "http://localhost:8086";
    let token = std::env::var("INFLUXDB2_TOKEN").unwrap();

    let client = influxdb2::Client::new(influx_url, org, token);

    let points = vec![
        CpuLoadShort {
            host: Some("server01".to_owned()),
            region: Some("us-west".to_owned()),
            value: 0.64,
            time: Utc::now().timestamp_nanos(),
        },
        CpuLoadShort {
            host: Some("server02".to_owned()),
            region: None,
            value: 0.64,
            time: Utc::now().timestamp_nanos(),
        },
    ];

    client.write(bucket, points)?;

    Ok(())
}
