fn main() -> Result<(), Box<dyn std::error::Error>> {
    let influx_url = "http://localhost:8888";
    let token = "some-token";

    let client = influxdb2::Client::new(influx_url, "org", token);

    if client.is_onboarding_allowed()? {
        println!(
            "{:?}",
            client
                .onboarding("some-user", "some-org", "some-bucket", None, None, None,)
                ?
        );
    }

    println!(
        "{:?}",
        client
            .post_setup_user(
                "some-new-user",
                "some-new-org",
                "some-new-bucket",
                None,
                None,
                None,
            )
            ?
    );

    Ok(())
}
