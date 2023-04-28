use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use mqtt::client::Client as MqttClient;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    process::exit,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

type AppState = Arc<Mutex<MqttClient>>;

#[derive(Serialize)]
struct SensorConfig<'a> {
    pub name: &'static str,
    pub unique_id: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<&'static str>,
    pub device: &'a HashMap<&'static str, &'static str>,
    pub state_topic: &'static str,
    pub unit_of_measurement: Option<&'static str>,
    pub state_class: &'static str,
}

#[derive(Deserialize)]
struct AppConfig {
    pub broker_address: String,
    pub client_id: String,
    pub username: String,
    pub password: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg: AppConfig = match tokio::fs::read_to_string("config.toml").await {
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

    let device = HashMap::from([
        ("identifiers", "ambw_mqtt"),
        ("manufacturer", "Ambient Weather"),
        ("model", "WS-2902"),
        ("name", "MQTT Weather Station"),
        ("via_device", "ambient_weather_mqtt"),
    ]);

    publish_sensor_config(
        &mqtt_client,
        "temperature",
        &SensorConfig {
            name: "Outside Temperature",
            unique_id: "ambw_mqtt_outside_temp",
            device_class: Some("temperature"),
            state_topic: "homeassistant/sensor/ambientWeather/temperature/state",
            unit_of_measurement: Some("°F"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "humidity",
        &SensorConfig {
            name: "Outside Humidity",
            unique_id: "ambw_mqtt_outside_hum",
            device_class: Some("humidity"),
            state_topic: "homeassistant/sensor/ambientWeather/humidity/state",
            unit_of_measurement: Some("%"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "dewPoint",
        &SensorConfig {
            name: "Outside Dew Point",
            unique_id: "ambw_mqtt_outside_dew",
            device_class: Some("temperature"),
            state_topic: "homeassistant/sensor/ambientWeather/dewPoint/state",
            unit_of_measurement: Some("°F"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "windChill",
        &SensorConfig {
            name: "Wind Chill",
            unique_id: "ambw_mqtt_wind_chill",
            device_class: Some("temperature"),
            state_topic: "homeassistant/sensor/ambientWeather/windChill/state",
            unit_of_measurement: Some("°F"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "windDir",
        &SensorConfig {
            name: "Wind Dir",
            unique_id: "ambw_mqtt_wind_dir",
            device_class: None,
            state_topic: "homeassistant/sensor/ambientWeather/windDir/state",
            unit_of_measurement: Some("°"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "windSpeed",
        &SensorConfig {
            name: "Wind Speed",
            unique_id: "ambw_mqtt_wind_speed",
            device_class: Some("wind_speed"),
            state_topic: "homeassistant/sensor/ambientWeather/windSpeed/state",
            unit_of_measurement: Some("mph"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "windGust",
        &SensorConfig {
            name: "Wind Gust",
            unique_id: "ambw_mqtt_wind_gust",
            device_class: Some("wind_speed"),
            state_topic: "homeassistant/sensor/ambientWeather/windGust/state",
            unit_of_measurement: Some("mph"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "rainHourly",
        &SensorConfig {
            name: "Hourly Rain Rate",
            unique_id: "ambw_mqtt_hourly_rain",
            device_class: Some("precipitation_intensity"),
            state_topic: "homeassistant/sensor/ambientWeather/rainHourly/state",
            unit_of_measurement: Some("in/h"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "rainDaily",
        &SensorConfig {
            name: "Daily Rain",
            unique_id: "ambw_mqtt_daily_rain",
            device_class: Some("precipitation"),
            state_topic: "homeassistant/sensor/ambientWeather/rainDaily/state",
            unit_of_measurement: Some("in"),
            state_class: "total_increasing",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "rainWeekly",
        &SensorConfig {
            name: "Weekly Rain",
            unique_id: "ambw_mqtt_weekly_rain",
            device_class: Some("precipitation"),
            state_topic: "homeassistant/sensor/ambientWeather/rainWeekly/state",
            unit_of_measurement: Some("in"),
            state_class: "total_increasing",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "rainMonthly",
        &SensorConfig {
            name: "Monthly Rain",
            unique_id: "ambw_mqtt_monthyl_rain",
            device_class: Some("precipitation"),
            state_topic: "homeassistant/sensor/ambientWeather/rainMonthly/state",
            unit_of_measurement: Some("in"),
            state_class: "total_increasing",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "rainLifetime",
        &SensorConfig {
            name: "Lifetime Rain",
            unique_id: "ambw_mqtt_lifetime_rain",
            device_class: Some("precipitation"),
            state_topic: "homeassistant/sensor/ambientWeather/rainLifetime/state",
            unit_of_measurement: Some("in"),
            state_class: "total_increasing",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "solarRadiation",
        &SensorConfig {
            name: "Solar Radiation",
            unique_id: "ambw_mqtt_solar_rad",
            device_class: Some("irradiance"),
            state_topic: "homeassistant/sensor/ambientWeather/solarRadiation/state",
            unit_of_measurement: Some("W/m²"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "UV",
        &SensorConfig {
            name: "UV Index",
            unique_id: "ambw_mqtt_uv",
            device_class: None,
            state_topic: "homeassistant/sensor/ambientWeather/UV/state",
            unit_of_measurement: None,
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "kitchenTemperature",
        &SensorConfig {
            name: "Kitchen Temperature",
            unique_id: "ambw_mqtt_indoor_temp",
            device_class: Some("temperature"),
            state_topic: "homeassistant/sensor/ambientWeather/kitchenTemperature/state",
            unit_of_measurement: Some("°F"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "kitchenHumidity",
        &SensorConfig {
            name: "Kitchen Humidity",
            unique_id: "ambw_mqtt_indoor_hum",
            device_class: Some("humidity"),
            state_topic: "homeassistant/sensor/ambientWeather/kitchenHumidity/state",
            unit_of_measurement: Some("%"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "pressure",
        &SensorConfig {
            name: "Outside Pressure",
            unique_id: "ambw_mqtt_abs_press",
            device_class: Some("atmospheric_pressure"),
            state_topic: "homeassistant/sensor/ambientWeather/pressure/state",
            unit_of_measurement: Some("hPa"),
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "relativePressure",
        &SensorConfig {
            name: "Outside Relative Pressure",
            unique_id: "ambw_mqtt_rel_press",
            device_class: Some("atmospheric_pressure"),
            state_topic: "homeassistant/sensor/ambientWeather/relativePressure/state",
            unit_of_measurement: Some("hPa"),
            state_class: "measurement",
            device: &device,
        },
    );

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
    publish_f32(&client, &params, "indoortempf", "kitchenTemperature", 1);
    publish_i32(&client, &params, "indoorhumidity", "kitchenHumidity");

    if let Some(val) = params.get("absbaromin") {
        match val.parse::<f32>() {
            Ok(inhg) => {
                let payload = format!("{:.1}", inhg * 33.86); //hPa
                info!(topic = "pressure", payload, "publishing");
                client.publish(
                    "homeassistant/sensor/ambientWeather/pressure/state",
                    &payload,
                    false,
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
                    false,
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
                    false,
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
                client.publish(
                    &format!("homeassistant/sensor/ambientWeather/{topic}/state"),
                    &payload,
                    false,
                );
            }
            Err(e) => {
                error!(%e, val, key, "unable to parse i32 from param");
            }
        }
    } else {
        error!(key, "missing value in params");
    }
}

fn publish_sensor_config(client: &MqttClient, topic: &str, config: &SensorConfig) {
    let payload = serde_json::to_string(config).unwrap();
    info!(topic, payload, "publishing config");
    client.publish(
        &format!("homeassistant/sensor/ambientWeather/{topic}/config"),
        &payload,
        true,
    );
}
