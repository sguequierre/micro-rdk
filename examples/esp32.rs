#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const SSID: &str = env!("MINI_RDK_WIFI_SSID");
#[allow(dead_code)]
#[cfg(not(feature = "qemu"))]
const PASS: &str = env!("MINI_RDK_WIFI_PASSWORD");

// Generated robot config during build process
include!(concat!(env!("OUT_DIR"), "/robot_secret.rs"));

#[cfg(all(not(feature = "qemu"), feature = "camera"))]
use crate::camera::Esp32Camera;

use anyhow::bail;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::task::notify;
#[cfg(feature = "qemu")]
use esp_idf_svc::eth::*;
#[cfg(feature = "qemu")]
use esp_idf_svc::eth::{EspEth, EthWait};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mdns::EspMdns;
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
#[cfg(not(feature = "qemu"))]
use esp_idf_svc::wifi::EspWifi;
#[cfg(not(feature = "qemu"))]
use esp_idf_sys::esp_wifi_set_ps;
use esp_idf_sys::vTaskDelay;
use esp_idf_sys::{self as _, TaskHandle_t}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use futures_lite::future::block_on;
use hyper::server::conn::Http;
use log::*;
use mini_rdk::esp32::exec::Esp32Executor;
use mini_rdk::esp32::grpc::GrpcServer;
use mini_rdk::esp32::robot::Esp32Robot;
use mini_rdk::esp32::robot::ResourceType;
use mini_rdk::esp32::robot_client::RobotClientConfig;
use mini_rdk::esp32::tcp::Esp32Listener;
use mini_rdk::esp32::tls::{Esp32Tls, Esp32TlsServerConfig};
use mini_rdk::proto::common::v1::ResourceName;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();
    let sys_loop_stack = EspSystemEventLoop::take().unwrap();
    let periph = Peripherals::take().unwrap();

    #[cfg(not(feature = "qemu"))]
    let robot = {
        use esp_idf_hal::adc::config::Config;
        use esp_idf_hal::adc::{self, AdcChannelDriver, AdcDriver, Atten11dB};
        use esp_idf_hal::gpio::PinDriver;
        use esp_idf_hal::ledc;
        use esp_idf_hal::ledc::config::TimerConfig;
        use esp_idf_hal::units::FromValueType;
        use mini_rdk::esp32::analog::Esp32AnalogReader;
        use mini_rdk::esp32::base::Esp32WheelBase;
        use mini_rdk::esp32::board::EspBoard;
        use mini_rdk::esp32::motor::MotorEsp32;
        #[cfg(feature = "camera")]
        let camera = {
            Esp32Camera::new();
            camera.setup()?;
            Arc::new(Mutex::new(camera))
        };
        // // let mut encoder = Esp32Encoder::new(
        // //     periph.pins.gpio15.into_input()?.degrade(),
        // //     periph.pins.gpio14.into_input()?.degrade(),
        // // );
        // // encoder.setup_pcnt()?;
        // // encoder.start()?;
        let tconf = TimerConfig::default().frequency(10.kHz().into());
        let timer = Arc::new(ledc::LedcTimerDriver::new(periph.ledc.timer0, &tconf).unwrap());
        let chan = ledc::LedcDriver::new(
            periph.ledc.channel0,
            timer.clone(),
            periph.pins.gpio14,
            &tconf,
        )?;
        let m1 = MotorEsp32::new(
            PinDriver::output(periph.pins.gpio33)?,
            PinDriver::output(periph.pins.gpio32)?,
            chan,
        );
        let chan2 = ledc::LedcDriver::new(
            periph.ledc.channel2,
            timer.clone(),
            periph.pins.gpio2,
            &tconf,
        )?;
        let m2 = MotorEsp32::new(
            PinDriver::output(periph.pins.gpio13)?,
            PinDriver::output(periph.pins.gpio12)?,
            chan2,
        );

        let pins = vec![PinDriver::output(periph.pins.gpio15)?];
        let adc1 = Rc::new(RefCell::new(AdcDriver::new(
            periph.adc1,
            &Config::new().calibration(true),
        )?));
        let chan: AdcChannelDriver<_, Atten11dB<adc::ADC1>> =
            AdcChannelDriver::new(periph.pins.gpio34)?;
        let r = Esp32AnalogReader::new("A1".to_string(), chan, adc1.clone());
        let chan: AdcChannelDriver<_, Atten11dB<adc::ADC1>> =
            AdcChannelDriver::new(periph.pins.gpio35)?;
        let r2 = Esp32AnalogReader::new("A2".to_string(), chan, adc1.clone());
        let b = EspBoard::new(
            pins,
            vec![Rc::new(RefCell::new(r)), Rc::new(RefCell::new(r2))],
        );
        let motor = Arc::new(Mutex::new(m1));
        let m2 = Arc::new(Mutex::new(m2));
        let board = Arc::new(Mutex::new(b));
        let base = Arc::new(Mutex::new(Esp32WheelBase::new(motor.clone(), m2.clone())));

        let mut res: mini_rdk::esp32::robot::ResourceMap = HashMap::with_capacity(5);
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "motor".to_string(),
                name: "m1".to_string(),
            },
            ResourceType::Motor(motor),
        );
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "motor".to_string(),
                name: "m2".to_string(),
            },
            ResourceType::Motor(m2),
        );
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "board".to_string(),
                name: "b".to_string(),
            },
            ResourceType::Board(board),
        );
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "base".to_string(),
                name: "base".to_string(),
            },
            ResourceType::Base(base),
        );
        #[cfg(feature = "camera")]
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "camera".to_string(),
                name: "c".to_string(),
            },
            ResourceType::Camera(camera),
        );
        Esp32Robot::new(res)
    };

    #[cfg(feature = "qemu")]
    let robot = {
        use mini_rdk::common::analog::FakeAnalogReader;
        use mini_rdk::common::base::FakeBase;
        use mini_rdk::common::board::FakeBoard;
        #[cfg(feature = "camera")]
        use mini_rdk::common::camera::FakeCamera;
        use mini_rdk::common::motor::FakeMotor;
        let motor = Arc::new(Mutex::new(FakeMotor::new()));
        let base = Arc::new(Mutex::new(FakeBase::new()));
        let board = Arc::new(Mutex::new(FakeBoard::new(vec![
            Rc::new(RefCell::new(FakeAnalogReader::new("A1".to_string(), 10))),
            Rc::new(RefCell::new(FakeAnalogReader::new("A2".to_string(), 20))),
        ])));
        #[cfg(feature = "camera")]
        let camera = Arc::new(Mutex::new(FakeCamera::new()));
        let mut res: mini_rdk::esp32::robot::ResourceMap = HashMap::with_capacity(1);
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "motor".to_string(),
                name: "m1".to_string(),
            },
            ResourceType::Motor(motor),
        );
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "board".to_string(),
                name: "b".to_string(),
            },
            ResourceType::Board(board),
        );
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "base".to_string(),
                name: "base".to_string(),
            },
            ResourceType::Base(base),
        );
        #[cfg(feature = "camera")]
        res.insert(
            ResourceName {
                namespace: "rdk".to_string(),
                r#type: "component".to_string(),
                subtype: "camera".to_string(),
                name: "c".to_string(),
            },
            ResourceType::Camera(camera),
        );
        Esp32Robot::new(res)
    };

    #[cfg(feature = "qemu")]
    let (ip, _eth) = {
        use std::net::Ipv4Addr;
        info!("creating eth object");
        let eth = eth_configure(
            &sys_loop_stack,
            Box::new(esp_idf_svc::eth::EspEth::wrap(EthDriver::new_openeth(
                periph.mac,
                sys_loop_stack.clone(),
            )?)?),
        )?;
        (Ipv4Addr::new(0, 0, 0, 0), eth)
    };
    {
        esp_idf_sys::esp!(unsafe {
            esp_idf_sys::esp_vfs_eventfd_register(&esp_idf_sys::esp_vfs_eventfd_config_t {
                max_fds: 5,
            })
        })?;
    }

    #[allow(clippy::redundant_clone)]
    #[cfg(not(feature = "qemu"))]
    let (ip, _wifi) = {
        let wifi = start_wifi(periph.modem, sys_loop_stack)?;
        (wifi.sta_netif().get_ip_info()?.ip, wifi)
    };

    let client_cfg = { RobotClientConfig::new(ROBOT_SECRET.to_string(), ROBOT_ID.to_string(), ip) };

    let hnd = match mini_rdk::esp32::robot_client::start(client_cfg) {
        Err(e) => {
            log::error!("couldn't start robot client {:?} will start the server", e);
            None
        }
        Ok(hnd) => Some(hnd),
    };
    // start mdns service
    let _mdms = {
        let mut mdns = EspMdns::take()?;
        mdns.set_hostname(ROBOT_NAME)?;
        mdns.set_instance_name(ROBOT_NAME)?;
        mdns.add_service(None, "_rpc", "_tcp", 80, &[])?;
        mdns
    };
    if let Err(e) = runserver(robot, hnd) {
        log::error!("robot server failed with error {:?}", e);
        return Err(e);
    }
    Ok(())
}

fn runserver(robot: Esp32Robot, client_handle: Option<TaskHandle_t>) -> anyhow::Result<()> {
    let cfg = {
        let cert = include_bytes!(concat!(env!("OUT_DIR"), "/ca.crt"));
        let key = include_bytes!(concat!(env!("OUT_DIR"), "/key.key"));
        Esp32TlsServerConfig::new(
            cert.as_ptr(),
            cert.len() as u32,
            key.as_ptr(),
            key.len() as u32,
        )
    };
    let tls = Box::new(Esp32Tls::new_server(&cfg));
    let address: SocketAddr = "0.0.0.0:80".parse().unwrap();
    let mut listener = Esp32Listener::new(address.into(), Some(tls))?;
    let exec = Esp32Executor::new();
    let srv = GrpcServer::new(Arc::new(Mutex::new(robot)));
    if let Some(hnd) = client_handle {
        if unsafe { notify(hnd, 1) } {
            log::info!("successfully notified client task");
            unsafe {
                vTaskDelay(1000);
            };
        } else {
            log::error!("failed to notity client task had handle {:?}", hnd);
        }
    } else {
        log::error!("no handle")
    }
    loop {
        let stream = listener.accept()?;
        block_on(exec.run(async {
            let err = Http::new()
                .with_executor(exec.clone())
                .http2_max_concurrent_streams(1)
                .serve_connection(stream, srv.clone())
                .await;
            if err.is_err() {
                log::error!("server error {}", err.err().unwrap());
            }
        }));
    }
}
#[cfg(feature = "qemu")]
fn eth_configure(
    sl_stack: &EspSystemEventLoop,
    mut eth: Box<EspEth<'static>>,
) -> anyhow::Result<Box<EspEth<'static>>> {
    use std::net::Ipv4Addr;

    eth.start()?;

    if !EthWait::new(eth.driver(), sl_stack)?
        .wait_with_timeout(Duration::from_secs(30), || eth.is_started().unwrap())
    {
        bail!("couldn't start eth driver")
    }

    if !EspNetifWait::new::<EspNetif>(eth.netif(), sl_stack)?
        .wait_with_timeout(Duration::from_secs(20), || {
            eth.netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        })
    {
        bail!("didn't get an ip")
    }
    let ip_info = eth.netif().get_ip_info()?;
    info!("ETH IP {:?}", ip_info);
    Ok(eth)
}

#[cfg(not(feature = "qemu"))]
fn start_wifi(
    modem: impl esp_idf_hal::peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sl_stack: EspSystemEventLoop,
) -> anyhow::Result<Box<EspWifi<'static>>> {
    use embedded_svc::wifi::{ClientConfiguration, Wifi};
    use esp_idf_svc::wifi::WifiWait;
    use std::net::Ipv4Addr;

    let mut wifi = Box::new(EspWifi::new(modem, sl_stack.clone(), None)?);

    info!("scanning");
    let aps = wifi.scan()?;
    let foundap = aps.into_iter().find(|x| x.ssid == SSID);

    let channel = if let Some(foundap) = foundap {
        info!("{} channel is {}", "Viam", foundap.channel);
        Some(foundap.channel)
    } else {
        None
    };
    let client_config = ClientConfiguration {
        ssid: SSID.into(),
        password: PASS.into(),
        channel,
        ..Default::default()
    };
    wifi.set_configuration(&embedded_svc::wifi::Configuration::Client(client_config))?; //&Configuration::Client(client_config)

    wifi.start()?;

    if !WifiWait::new(&sl_stack)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        bail!("couldn't start wifi")
    }

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sl_stack)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("wifi couldn't connect")
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    esp_idf_sys::esp!(unsafe { esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE) })?;

    Ok(wifi)
}