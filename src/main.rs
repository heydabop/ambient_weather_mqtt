use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use mqtt::client::Client as MqttClient;
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    process::exit,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

type AppState = Arc<Mutex<MqttClient>>;

#[derive(Deserialize)]
struct Config {
    pub broker_address: String,
    pub client_id: String,
    pub username: String,
    pub password: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg: Config = match tokio::fs::read_to_string("config.toml").await {
        Ok(s) => match toml::from_str(&s) {
            Ok(t) => t,
            Err(e) => {
                error!(%e, "unable to parse config.toml");
                exit(1);
            }
        },
        Err(e) => {
            error!(%e, "unable to load config.toml");
            exit(1);
        }
    };

    let mut mqtt_client = MqttClient::new(&cfg.client_id, &cfg.username, &cfg.password, 60);
    if let Err(e) = mqtt_client.connect(&cfg.broker_address) {
        error!(%e, "unable to connect to MQTT broker");
        exit(1);
    }

    mqtt_client.publish("homeassistant/sensor/ambientWeather/temperature/config", r#"{"name":"Outside Temperature","device_class":"temperature","state_topic":"homeassistant/sensor/ambientWeather/temperature/state","unit_of_measurement":"°F","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/humidity/config", r#"{"name":"Outside Humidity","device_class":"humidity","state_topic":"homeassistant/sensor/ambientWeather/humidity/state","unit_of_measurement":"%","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/dewPoint/config", r#"{"name":"Outside Dew Point","device_class":"temperature","state_topic":"homeassistant/sensor/ambientWeather/dewPoint/state","unit_of_measurement":"°F","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/windChill/config", r#"{"name":"Wind Chill","device_class":"temperature","state_topic":"homeassistant/sensor/ambientWeather/windChill/state","unit_of_measurement":"°F","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/windDir/config", r#"{"name":"Wind Dir","state_topic":"homeassistant/sensor/ambientWeather/windDir/state","unit_of_measurement":"°"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/windSpeed/config", r#"{"name":"Wind Speed","device_class":"wind_speed","state_topic":"homeassistant/sensor/ambientWeather/windSpeed/state","unit_of_measurement":"mph","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/windGust/config", r#"{"name":"Wind Gust","device_class":"wind_speed","state_topic":"homeassistant/sensor/ambientWeather/windGust/state","unit_of_measurement":"mph","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/rainHourly/config", r#"{"name":"Hourly Rain Rate","device_class":"precipitation_intensity","state_topic":"homeassistant/sensor/ambientWeather/rainHourly/state","unit_of_measurement":"in/h","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/rainDaily/config", r#"{"name":"Daily Rain","device_class":"precipitation","state_topic":"homeassistant/sensor/ambientWeather/rainDaily/state","unit_of_measurement":"in","state_class":"total_increasing"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/rainWeekly/config", r#"{"name":"Weekly Rain","device_class":"precipitation","state_topic":"homeassistant/sensor/ambientWeather/rainWeekly/state","unit_of_measurement":"in","state_class":"total_increasing"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/rainMonthly/config", r#"{"name":"Monthly Rain","device_class":"precipitation","state_topic":"homeassistant/sensor/ambientWeather/rainMonthly/state","unit_of_measurement":"in","state_class":"total_increasing"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/rainLifetime/config", r#"{"name":"Lifetime Rain","device_class":"precipitation","state_topic":"homeassistant/sensor/ambientWeather/rainLifetime/state","unit_of_measurement":"in","state_class":"total_increasing"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/solarRadiation/config", r#"{"name":"Solar Radiation","device_class":"irradiance","state_topic":"homeassistant/sensor/ambientWeather/solarRadiation/state","unit_of_measurement":"W/m²","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/UV/config", r#"{"name":"UV Index","state_topic":"homeassistant/sensor/ambientWeather/UV/state","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/kitchenTemperature/config", r#"{"name":"Kitchen Temperature","device_class":"temperature","state_topic":"homeassistant/sensor/ambientWeather/kitchenTemperature/state","unit_of_measurement":"°F","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/kitchenHumidity/config", r#"{"name":"Kitchen Humidity","device_class":"humidity","state_topic":"homeassistant/sensor/ambientWeather/kitchenHumidity/state","unit_of_measurement":"%","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/pressure/config", r#"{"name":"Outside Pressure","device_class":"pressure","state_topic":"homeassistant/sensor/ambientWeather/pressure/state","unit_of_measurement":"hPa","state_class":"measurement"}"#);
    mqtt_client.publish("homeassistant/sensor/ambientWeather/relativePressure/config", r#"{"name":"Outside Relative Pressure","device_class":"pressure","state_topic":"homeassistant/sensor/ambientWeather/relativePressure/state","unit_of_measurement":"hPa","state_class":"measurement"}"#);

    let state = Arc::new(Mutex::new(mqtt_client));

    let app = Router::new()
        .route("/update_weather", get(handle_weather_update))
        .with_state(state);

    let addr = SocketAddr::from(([192, 168, 1, 2], 8090));
    info!("Listening on {}", addr);
    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        error!(error = %e, "Error running axum server");
    }
}

#[allow(clippy::unused_async)]
pub async fn handle_weather_update(
    State(mqtt_client): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    info!(?params, "incoming payload");

    if params.get("ID") != Some(&String::from("local"))
        || params.get("PASSWORD") != Some(&String::from("key"))
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let client = mqtt_client.lock().map_err(|e| {
        error!(%e, "unable to lock MQTT client mutex");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    publish_f32(&client, &params, "tempf", "temperature", 1);
    publish_i32(&client, &params, "humidity", "humidity");
    publish_f32(&client, &params, "dewptf", "dewPoint", 1);
    publish_f32(&client, &params, "windchillf", "windChill", 1);
    publish_i32(&client, &params, "winddir", "windDir");
    publish_f32(&client, &params, "windspeedmph", "windSpeed", 2);
    publish_f32(&client, &params, "windgustmph", "windGust", 2);
    publish_f32(&client, &params, "rainin", "rainHourly", 3);
    publish_f32(&client, &params, "dailyrainin", "rainDaily", 3);
    publish_f32(&client, &params, "weeklyrainin", "rainWeekly", 3);
    publish_f32(&client, &params, "monthlyrainin", "rainMonthly", 3);
    publish_f32(&client, &params, "totalrainin", "rainLifetime", 3);
    publish_f32(&client, &params, "solarradiation", "solarRadiation", 1);
    publish_i32(&client, &params, "UV", "UV");
    publish_i32(&client, &params, "indoorhumidity", "kitchenHumidity");

    if let Some(val) = params.get("absbaromin") {
        match val.parse::<f32>() {
            Ok(inhg) => {
                let payload = format!("{:.1}", inhg * 33.86); //hPa
                info!(topic = "pressure", payload, "publishing");
                client.publish(
                    "homeassistant/sensor/ambientWeather/pressure/state",
                    &payload,
                )
            }
            Err(e) => {
                error!(%e, val, key = "absbaromin", "unable to parse f32 from param");
            }
        }
    } else {
        error!(key = "absbaromin", "missing value in params");
    }

    if let Some(val) = params.get("baromin") {
        match val.parse::<f32>() {
            Ok(inhg) => {
                let payload = format!("{:.1}", inhg * 33.86); //hPa
                info!(topic = "relativePressure", payload, "publishing");
                client.publish(
                    "homeassistant/sensor/ambientWeather/relativePressure/state",
                    &payload,
                )
            }
            Err(e) => {
                error!(%e, val, key = "baromin", "unable to parse f32 from param");
            }
        }
    } else {
        error!(key = "baromin", "missing value in params");
    }

    Ok(StatusCode::OK)
}

fn publish_f32(
    client: &MqttClient,
    params: &HashMap<String, String>,
    key: &str,
    topic: &str,
    precision: usize,
) {
    if let Some(val) = params.get(key) {
        match val.parse::<f32>() {
            Ok(parsed) => {
                let payload = format!("{:.1$}", parsed, precision);
                info!(topic, payload, "publishing");
                client.publish(
                    &format!("homeassistant/sensor/ambientWeather/{topic}/state"),
                    &payload,
                )
            }
            Err(e) => {
                error!(%e, val, key, "unable to parse f32 from param");
            }
        }
    } else {
        error!(key, "missing value in params");
    }
}

fn publish_i32(client: &MqttClient, params: &HashMap<String, String>, key: &str, topic: &str) {
    if let Some(val) = params.get(key) {
        match val.parse::<i32>() {
            Ok(parsed) => {
                let payload = parsed.to_string();
                info!(topic, payload, "publishing");
                client.publish(topic, &payload);
            }
            Err(e) => {
                error!(%e, val, key, "unable to parse i32 from param");
            }
        }
    } else {
        error!(key, "missing value in params");
    }
}
