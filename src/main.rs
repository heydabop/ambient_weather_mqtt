#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use mqtt::client::Client as MqttClient;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    process::exit,
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info};

type AppState = Arc<Mutex<MqttClient>>;

#[derive(Serialize)]
struct SensorConfig<'a> {
    pub name: &'static str,
    pub unique_id: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<&'static str>,
    pub device: &'a HashMap<&'static str, &'static str>,
    pub state_topic: &'static str,
    pub unit_of_measurement: &'static str,
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
            unit_of_measurement: "°F",
            state_class: "measurement",
            device: &device,
        },
    );
    publish_sensor_config(
        &mqtt_client,
        "feelsLike",
        &SensorConfig {
            name: "Outside Feels Like",
            unique_id: "ambw_mqtt_outside_feels",
            device_class: Some("temperature"),
            state_topic: "homeassistant/sensor/ambientWeather/feelsLike/state",
            unit_of_measurement: "°F",
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
            unit_of_measurement: "%",
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
            unit_of_measurement: "°F",
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
            unit_of_measurement: "°F",
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
            unit_of_measurement: "°",
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
            unit_of_measurement: "mph",
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
            unit_of_measurement: "mph",
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
            unit_of_measurement: "in/h",
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
            unit_of_measurement: "in",
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
            unit_of_measurement: "in",
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
            unit_of_measurement: "in",
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
            unit_of_measurement: "in",
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
            unit_of_measurement: "W/m²",
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
            unit_of_measurement: "Index",
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
            unit_of_measurement: "°F",
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
            unit_of_measurement: "%",
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
            unit_of_measurement: "hPa",
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
            unit_of_measurement: "hPa",
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

#[allow(
    clippy::unused_async,
    clippy::implicit_hasher,
    clippy::missing_errors_doc
)]
pub async fn handle_weather_update(
    State(mqtt_client): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    debug!(?params, "incoming payload");

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
                debug!(topic = "pressure", payload, "publishing");
                client.publish(
                    "homeassistant/sensor/ambientWeather/pressure/state",
                    &payload,
                    false,
                );
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
                debug!(topic = "relativePressure", payload, "publishing");
                client.publish(
                    "homeassistant/sensor/ambientWeather/relativePressure/state",
                    &payload,
                    false,
                );
            }
            Err(e) => {
                error!(%e, val, key = "baromin", "unable to parse f32 from param");
            }
        }
    } else {
        error!(key = "baromin", "missing value in params");
    }

    if let Some(Ok(temp_f)) = params.get("tempf").map(|t| t.parse::<f64>()) {
        if let Some(Ok(rh)) = params.get("humidity").map(|w| w.parse::<f64>()) {
            let heat_index_f = {
                if temp_f < 80.0 {
                    temp_f
                } else {
                    let steadman = 0.5 * (temp_f + 61.0 + (temp_f - 68.0) * 1.2 + rh * 0.094);
                    let s_avg = (temp_f + steadman) / 2.0;
                    if s_avg < 80.0 {
                        steadman
                    } else {
                        let rothfusz = -42.379 + 2.049_015_23 * temp_f + 10.143_331_27 * rh
                            - 0.224_755_41 * temp_f * rh
                            - 0.006_837_83 * temp_f * temp_f
                            - 0.054_817_17 * rh * rh
                            + 0.001_228_74 * temp_f * temp_f * rh
                            + 0.000_852_82 * temp_f * rh * rh
                            - 0.000_001_99 * temp_f * temp_f * rh * rh;
                        if rh < 13.0 && temp_f > 80.0 && temp_f < 112.0 {
                            rothfusz
                                - ((13.0 - rh) / 4.0)
                                    * ((17.0 - (temp_f - 95.0).abs()) / 17.0).sqrt()
                        } else if rh > 85.0 && temp_f > 80.0 && temp_f < 87.0 {
                            rothfusz + ((rh - 85.0) / 10.0) * ((87.0 - temp_f) / 5.0)
                        } else {
                            rothfusz
                        }
                    }
                }
            };
            let payload = format!("{heat_index_f:.1}");
            debug!(topic = "feelsLike", payload, "publishing");
            client.publish(
                "homeassistant/sensor/ambientWeather/feelsLike/state",
                &payload,
                false,
            );
        }
    }

    if let Some(Ok(temp_f)) = params.get("tempf").map(|t| t.parse::<f32>()) {
        if let Some(Ok(wind_mph)) = params.get("windspeedmph").map(|w| w.parse::<f32>()) {
            if let Some(reported_wind_chill_f) = params.get("windchillf") {
                if temp_f <= 50.0 && wind_mph > 3.0 {
                    let wind_chill_f = 35.74 + (0.6215 * temp_f) - (35.75 * wind_mph.powf(0.16))
                        + (0.4275 * temp_f * wind_mph.powf(0.16));
                    debug!(
                        computed_wind_chill = format!("{wind_chill_f:.1}"),
                        reported_wind_chill = reported_wind_chill_f
                    );
                }
            }
        }
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
                let payload = format!("{parsed:.precision$}");
                debug!(topic, payload, "publishing");
                client.publish(
                    &format!("homeassistant/sensor/ambientWeather/{topic}/state"),
                    &payload,
                    false,
                );
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
                debug!(topic, payload, "publishing");
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
    debug!(topic, payload, "publishing config");
    client.publish(
        &format!("homeassistant/sensor/ambientWeather/{topic}/config"),
        &payload,
        true,
    );
}
