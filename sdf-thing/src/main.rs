use esp_idf_hal::temp_sensor::{TempSensorConfig, TempSensorDriver};

use anyhow::anyhow;
use serde::Deserialize;
use shtcx::{shtc3, PowerMode};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
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
struct ConfigurationData {
    device_name: Option<String>,
    unit: Option<String>,
}

fn create_snapshot_message(
    ip_address: String,
    device_name: String,
    unit: String,
) -> anyhow::Result<SdfMessage> {
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
        .sdf_instance(
            SdfInstanceBuilder::default()
                .thing_id("urn:uuid:b38acf9d-493c-408c-90bf-868c1f5326d4")
                .sdf_context(HashMap::from_iter(vec![
                    (
                        "ipAddress".to_string(),
                        serde_json::Value::String(ip_address),
                    ),
                    (
                        "deviceName".to_string(),
                        serde_json::Value::String(device_name.to_string()),
                    ),
                    (
                        "unit".to_string(),
                        serde_json::Value::String(unit.to_string()),
                    ),
                ]))
                .build()?,
        )
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
    let device_id = sht.device_identifier().unwrap();
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

    let mut device_name = "CoAP Sensor";

    let mut unit = "Cel";

    let mut snapshot_message = create_snapshot_message(
        device_name.to_string(),
        device_name.to_string(),
        device_name.to_string(),
    )
    .unwrap();

    let snapshot_resource = CoapResource::new(".well-known/sdf/instance", (), false);

    snapshot_resource.set_method_handler(
        CoapRequestCode::Get,
        Some(CoapRequestHandler::new(
            move |_: &mut (),
                  session: &mut CoapServerSession,
                  _: &CoapRequest,
                  mut response: CoapResponse| {
                let info_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

                // TODO: Fahrenheit would be C x 9/5 + 32

                // property.insert("ipAddress".to_string(), info_info.ip.to_string().into());

                response.set_code(CoapResponseCode::Content);
                response.set_content_format(Some(9001));

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

                            if let Some(new_device_name) = &configuration_data.device_name {
                                println!("{new_device_name}");
                                // device_name = new_device_name.as_str();
                            }

                            if let Some(new_unit) = configuration_data.unit {
                                println!("{new_unit}");
                                // unit = new_unit.as_str();
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

                let temp_val = temp_sensor
                    .lock()
                    .unwrap()
                    .get_measurement_result()
                    .unwrap()
                    .temperature
                    .as_degrees_celsius();

                // TODO: Fahrenheit would be C x 9/5 + 32

                let json = format!("{temp_val:.2}");
                let data = Vec::<u8>::from(json.as_bytes());
                response.set_data(Some(data));

                response.set_code(CoapResponseCode::Content);
                response.set_content_format(Some(CoapContentFormat::Json as u16));

                session.send(response).expect("Unable to send response");
            },
        )),
    );

    let _sdf_instance = SdfInstanceBuilder::default().build();

    // Add the resource to the context.
    context.add_resource(resource);
    loop {
        // process IO in a loop...
        if let Err(e) = context.do_io(None) {
            break;
        }
        // ...until we want to shut down.
    }
    // Properly shut down, completing outstanding IO requests and properly closing sessions.
    context.shutdown(Some(Duration::from_secs(0))).unwrap();

    Ok(())
}
