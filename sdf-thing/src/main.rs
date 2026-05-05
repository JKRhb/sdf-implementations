use esp_idf_hal::temp_sensor::{TempSensorConfig, TempSensorDriver};

use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Number;
use shtcx::{shtc3, PowerMode};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    thread::sleep,
    time::Duration,
};
use uuid::Uuid;

use esp_idf_svc::{
    eventloop::EspEventLoop,
    hal::prelude::*,
    hal::{
        i2c::{I2cConfig, I2cDriver},
        prelude::Peripherals,
    },
    sntp::SyncStatus,
    wifi::{AuthMethod, BlockingWifi, EspWifi},
};
use libcoap_rs::crypto::psk::ServerPskContextBuilder;
use libcoap_rs::{crypto::psk::PskKey, types::CoapProtocol};
use libcoap_rs::{
    message::{CoapMessageCommon, CoapRequest, CoapResponse},
    protocol::{CoapRequestCode, CoapResponseCode},
    session::{CoapServerSession, CoapSessionCommon},
    CoapContext, CoapRequestHandler, CoapResource,
};

use sdf_data_structures::instance::{
    InfoBlockBuilder, SdfInstanceBuilder, SdfInstanceOfBuilder, SdfMessage, SdfMessageBuilder,
};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    dtls_identity: &'static str,
    #[default("")]
    dtls_psk: &'static str,
}

#[derive(Deserialize)]
enum Unit {
    /// Celcius
    Cel,

    /// Fahrenheit
    F,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Cel => f.write_str("Cel"),
            Unit::F => f.write_str("F"),
        }
    }
}

#[derive(Deserialize)]
struct ConfigurationData {
    device_name: Option<String>,
    unit: Option<Unit>,
}

static _SDF_MESSAGE: OnceLock<Mutex<SdfMessage>> = OnceLock::new();

static UNIT: OnceLock<Mutex<String>> = OnceLock::new();

static IP_ADDRESS: OnceLock<Mutex<String>> = OnceLock::new();

static DEVICE_NAME: OnceLock<Mutex<String>> = OnceLock::new();

fn create_snapshot_message(temperature: Option<f32>) -> anyhow::Result<SdfMessage> {
    let unit = UNIT.get().unwrap().lock().unwrap().clone();
    let ip_address = IP_ADDRESS.get().unwrap().lock().unwrap().clone();
    let device_name = DEVICE_NAME.get().unwrap().lock().unwrap().clone();

    let mut binding = SdfInstanceBuilder::default();
    let mut sdf_instance_builder = binding
        .thing_id("urn:uuid:b38acf9d-493c-408c-90bf-868c1f5326d4")
        .sdf_context([
            (
                "ipAddress".to_string(),
                serde_json::Value::String(ip_address),
            ),
            (
                "deviceName".to_string(),
                serde_json::Value::String(device_name.to_string()),
            ),
            ("unit".to_string(), serde_json::Value::String(unit)),
        ]);

    if let Some(temperature) = temperature {
        sdf_instance_builder = sdf_instance_builder.sdf_property([(
            "temperature".to_string(),
            serde_json::Value::Number(Number::from_f64(temperature as f64).unwrap()),
        )]);
    }

    Ok(SdfMessageBuilder::default()
        .info(
            InfoBlockBuilder::default()
                .title("CoAP Temperature Sensor No. 2")
                .description("A temperature sensor that uses CoAP.")
                .message_id(Uuid::new_v4())
                .build()?,
        )
        .namespace(HashMap::from_iter(vec![(
            "sensors".to_string(),
            "https://sdf-repository.org/sdf/sensor".to_string(),
        )]))
        .default_namespace("sensors")
        .sdf_instance_of(
            SdfInstanceOfBuilder::default()
                .entry_point("#/sdfObject/envSensor")
                .lineage("foobar")
                .min_version("1.1.0")
                .build()?,
        )
        .sdf_instance(sdf_instance_builder.build()?)
        .build()?)
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let event_loop = EspEventLoop::take()?;

    let peripherals = Peripherals::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, event_loop.clone(), None)?,
        event_loop.clone(),
    )?;

    wifi.start()?;

    let app_config = CONFIG;

    let wifi_cfg =
        esp_idf_svc::wifi::Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
            ssid: app_config
                .wifi_ssid
                .try_into()
                .map_err(|_e| anyhow!("unable to parse Wifi SSID"))?,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: app_config
                .wifi_psk
                .try_into()
                .map_err(|_e| anyhow!("unable to parse Wifi password"))?,
            channel: None,
            ..Default::default()
        });
    log::info!("{:?}", wifi.get_configuration()?);

    wifi.set_configuration(&wifi_cfg)?;
    wifi.connect()?;

    wifi.wait_netif_up()?;

    let sntp = esp_idf_svc::sntp::EspSntp::new_default()?;
    log::info!("waiting for time synchronization...");
    while sntp.get_sync_status() != SyncStatus::Completed {
        sleep(Duration::from_secs(1));
    }
    log::info!("time synchronized");

    let iface = wifi.wifi().sta_netif();

    log::warn!("Network Information: {:?}", iface.get_ip_info()?);

    let mut context = CoapContext::new()?;

    let default_key = PskKey::new(
        Some(app_config.dtls_identity),
        app_config.dtls_psk.as_bytes().to_vec(),
    );
    let psk_context = ServerPskContextBuilder::new(default_key).build();
    context.set_psk_context(psk_context)?;

    context.add_endpoint_udp("0.0.0.0:5683".parse()?)?;
    context.add_endpoint_dtls("0.0.0.0:5684".parse()?)?;

    unsafe {
        libcoap_rs::sys::coap_set_log_level(libcoap_rs::sys::coap_log_t_COAP_LOG_INFO);
        libcoap_rs::sys::coap_dtls_set_log_level(libcoap_rs::sys::coap_log_t_COAP_LOG_INFO);
    }

    // Initialize temperature sensor
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;
    let i2c = peripherals.i2c0;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
    let mut sht = shtc3(i2c);
    let _device_id = sht.device_identifier().unwrap();
    let sensor_main = Arc::new(Mutex::new(sht));
    let sensor = sensor_main.clone();
    sensor
        .lock()
        .unwrap()
        .start_measurement(PowerMode::NormalMode)
        .unwrap();

    let cfg = TempSensorConfig::default();
    let mut temp = TempSensorDriver::new(&cfg, peripherals.temp_sensor)?;
    temp.enable()?;

    DEVICE_NAME.set("CoAP Sensor".to_string().into()).unwrap();

    let snapshot_resource = CoapResource::new(".well-known/sdf/instance", (), false);

    let temp_sensor = sensor.clone();

    let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

    IP_ADDRESS.set(ip_info.ip.to_string().into()).unwrap();

    snapshot_resource.set_method_handler(
        CoapRequestCode::Get,
        Some(CoapRequestHandler::new(
            move |_: &mut (),
                  session: &mut CoapServerSession,
                  _: &CoapRequest,
                  mut response: CoapResponse| {
                let mut temp_val = temp_sensor
                    .lock()
                    .unwrap()
                    .get_measurement_result()
                    .unwrap()
                    .temperature
                    .as_degrees_celsius();

                if &*UNIT.get().unwrap().lock().unwrap() == "F" {
                    temp_val = temp_val * 9.0 / 5.0 + 32.0;
                }

                response.set_code(CoapResponseCode::Content);
                response.set_content_format(Some(9001));

                let snapshot_message = create_snapshot_message(Some(temp_val)).unwrap();

                let json = serde_json::to_string(&snapshot_message).unwrap();
                let data = Vec::<u8>::from(json.as_bytes());
                response.set_data(Some(data));

                session.send(response).expect("Unable to send response");
            },
        )),
    );

    snapshot_resource.set_method_handler(
        CoapRequestCode::Post,
        Some(CoapRequestHandler::new(
            move |_: &mut (),
                  session: &mut CoapServerSession,
                  request: &CoapRequest,
                  mut response: CoapResponse| {
                if session.proto() != CoapProtocol::Dtls {
                    response.set_code(CoapResponseCode::NotFound);
                } else if let Some(data) = request.data() {
                    if let Ok(sdf_message) = serde_json::from_slice::<SdfMessage>(data) {
                        let context_definitions = sdf_message.sdf_instance.sdf_context;

                        if let Some(context_definitions) = context_definitions {
                            let configuration_data = serde_json::from_value::<ConfigurationData>(
                                serde_json::to_value(context_definitions).unwrap(),
                            )
                            .unwrap();

                            if let Some(new_device_name) = configuration_data.device_name {
                                {
                                    let mut device_name =
                                        DEVICE_NAME.get().unwrap().lock().unwrap();

                                    device_name.clear();

                                    device_name.push_str(&new_device_name);
                                }
                            }

                            if let Some(new_unit) = configuration_data.unit {
                                {
                                    let mut unit = UNIT.get().unwrap().lock().unwrap();

                                    unit.clear();

                                    unit.push_str(&new_unit.to_string());
                                }
                            }
                        }

                        response.set_code(CoapResponseCode::Changed);
                    } else {
                        response.set_code(CoapResponseCode::BadRequest);
                    }
                } else {
                    response.set_code(CoapResponseCode::BadRequest);
                }

                session.send(response).expect("Unable to send response");
            },
        )),
    );

    context.add_resource(snapshot_resource);

    let resource = CoapResource::new("temperature", (), false);
    let temp_sensor = sensor.clone();

    resource.set_method_handler(
        CoapRequestCode::Get,
        Some(CoapRequestHandler::new(
            move |_: &mut (),
                  session: &mut CoapServerSession,
                  _: &CoapRequest,
                  mut response: CoapResponse| {
                use libcoap_rs::protocol::CoapContentFormat;

                let mut temp_val = temp_sensor
                    .lock()
                    .unwrap()
                    .get_measurement_result()
                    .unwrap()
                    .temperature
                    .as_degrees_celsius();

                if &*UNIT.get().unwrap().lock().unwrap() == "F" {
                    temp_val = temp_val * 9.0 / 5.0 + 32.0;
                }

                let json = format!("{temp_val:.2}");
                let data = Vec::<u8>::from(json.as_bytes());
                response.set_data(Some(data));

                response.set_code(CoapResponseCode::Content);
                response.set_content_format(Some(CoapContentFormat::Json as u16));

                session.send(response).expect("Unable to send response");
            },
        )),
    );

    context.add_resource(resource);
    loop {
        if let Err(_) = context.do_io(None) {
            break;
        }
    }
    context.shutdown(Some(Duration::from_secs(0))).unwrap();

    Ok(())
}
