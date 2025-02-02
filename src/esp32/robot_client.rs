#![allow(dead_code)]
use crate::{
    esp32::exec::Esp32Executor,
    esp32::tcp::Esp32Stream,
    esp32::tls::Esp32Tls,
    proto::{
        app::v1::{AgentInfo, ConfigRequest, ConfigResponse},
        rpc::v1::{AuthenticateRequest, AuthenticateResponse, Credentials},
    },
};
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use esp_idf_hal::task::{notify, wait_notification};
use esp_idf_sys::{vTaskDelete, xTaskCreatePinnedToCore, xTaskGetCurrentTaskHandle, TaskHandle_t};
use futures_lite::future::block_on;
use h2::client::{handshake, SendRequest};
use hyper::{Method, Request};
use prost::Message;
use smol::Task;
use std::{ffi::c_void, net::Ipv4Addr, time::Duration};

/// Robot client to interface with app.viam.com
struct RobotClient<'a> {
    /// a local executor to spawn future
    exec: Esp32Executor<'a>,
    /// an HTTP2 stream to a server
    h2: SendRequest<Bytes>,
    /// an connection to a server
    #[allow(dead_code)]
    http2_connection: Task<()>,
    /// a jwt string for further grpc requests
    jwt: Option<String>,
    config: &'a Box<RobotClientConfig>,
}

pub struct RobotClientConfig {
    robot_secret: String,
    robot_id: String,
    ip: Ipv4Addr,
    main_handle: Option<TaskHandle_t>,
}

impl RobotClientConfig {
    pub fn new(robot_secret: String, robot_id: String, ip: Ipv4Addr) -> Self {
        RobotClientConfig {
            robot_secret,
            robot_id,
            ip,
            main_handle: None,
        }
    }
    pub fn set_main_handle(&mut self, hnd: TaskHandle_t) {
        self.main_handle = Some(hnd)
    }
}

static CLIENT_TASK: &[u8] = b"client\0";

impl<'a> RobotClient<'a> {
    /// Create a new robot client
    fn new(
        exec: Esp32Executor<'a>,
        h2: SendRequest<Bytes>,
        http2_connection: Task<()>,
        config: &'a Box<RobotClientConfig>,
    ) -> Self {
        RobotClient {
            exec,
            h2,
            http2_connection,
            jwt: None,
            config,
        }
    }

    /// Make a request to app.viam.com
    fn build_request(&self, path: &str) -> Result<Request<()>> {
        let mut uri = "https://app.viam.com:443".to_owned();
        uri.push_str(path);

        let mut r = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/grpc")
            .header("te", "trailers")
            .header("user-agent", "esp32");

        if let Some(jwt) = &self.jwt {
            r = r.header("authorization", jwt.clone());
        };
        r.body(())
            .map_err(|e| anyhow::anyhow!("cannot build request {}", e))
    }

    /// read the robot config from the cloud
    fn read_config(&mut self) -> Result<()> {
        let r = self.build_request("/viam.app.v1.RobotService/Config")?;

        let agent = AgentInfo {
            os: "esp32".to_string(),
            host: "esp32".to_string(),
            ips: vec![self.config.ip.to_string()],
            version: "0.0.2".to_string(),
            git_revision: "".to_string(),
        };

        let req = ConfigRequest {
            agent_info: Some(agent),
            id: self.config.robot_id.clone(),
        };

        let body: Bytes = {
            let mut buf = BytesMut::with_capacity(req.encoded_len() + 5);

            buf.put_u8(0);
            buf.put_u32(req.encoded_len().try_into()?);

            let mut msg = buf.split_off(5);
            req.encode(&mut msg)?;
            buf.unsplit(msg);
            buf.into()
        };

        let mut r = self.send_request(r, body)?;
        let r = r.split_off(5);
        // for now we only read the config
        let _r = ConfigResponse::decode(r)?;
        log::info!("cfg {:?}", _r);

        Ok(())
    }

    /// get a JWT token from app.viam.com
    fn request_jwt_token(&mut self) -> Result<()> {
        let r = self.build_request("/proto.rpc.v1.AuthService/Authenticate")?;
        let body: Bytes = {
            let cred = Credentials {
                r#type: "robot-secret".to_string(),
                payload: self.config.robot_secret.clone(),
            };

            let req = AuthenticateRequest {
                entity: self.config.robot_id.clone(),
                credentials: Some(cred),
            };

            let mut buf = BytesMut::with_capacity(req.encoded_len() + 5);

            buf.put_u8(0);
            buf.put_u32(req.encoded_len().try_into()?);

            let mut msg = buf.split_off(5);
            req.encode(&mut msg)?;
            buf.unsplit(msg);

            buf.into()
        };

        let mut r = self.send_request(r, body)?;
        let r = r.split_off(5);
        let r = AuthenticateResponse::decode(r)?;

        self.jwt = Some(format!("Bearer {}", r.access_token));

        Ok(())
    }

    /// send a grpc request
    fn send_request(&mut self, r: Request<()>, body: Bytes) -> Result<Bytes> {
        let h2 = self.h2.clone();
        // verify if the server can accept a new HTTP2 stream
        let mut h2 = block_on(self.exec.run(async { h2.ready().await }))?;

        // send the header and let the server know more data are coming
        let (response, mut send) = h2.send_request(r, false)?;
        // send the body of the request and let the server know we have nothing else to send
        send.send_data(body, true)?;

        let (part, mut body) = block_on(self.exec.run(async { response.await }))?.into_parts();
        log::info!("parts received {:?}", part);

        let mut response_buf = BytesMut::with_capacity(1024);
        // TODO read the first 5 bytes so we know how much data to expect and we can allocate appropriately
        while let Some(chunk) = block_on(self.exec.run(async { body.data().await })) {
            let chunk = chunk?;
            response_buf.put_slice(&chunk);
            let _ = body.flow_control().release_capacity(chunk.len());
        }

        let _ = block_on(self.exec.run(async { body.trailers().await }));

        self.h2 = h2;

        Ok(response_buf.into())
    }
}

/// start the robot client
pub fn start(ip: RobotClientConfig) -> Result<TaskHandle_t> {
    log::info!("starting up robot client");
    let ip = Box::into_raw(Box::new(ip));
    let mut hnd: TaskHandle_t = std::ptr::null_mut();
    let ret = unsafe {
        xTaskCreatePinnedToCore(
            Some(client_entry),                // C ABI compatible entry function
            CLIENT_TASK.as_ptr() as *const i8, // task name
            8192 * 3,                          // stack size
            ip as *mut c_void,                 // pass ip as argument
            20,                                // priority (low)
            &mut hnd,                          // we don't store the handle
            0,                                 // run it on core 0
        )
    };
    if ret != 1 {
        return Err(anyhow::anyhow!("wasn't able to create the client task"));
    }
    log::error!("got handle {:?}", hnd);
    Ok(hnd)
}

/// client main loop
fn clientloop(config: &Box<RobotClientConfig>) -> Result<()> {
    let mut tls = Box::new(Esp32Tls::new_client());
    let conn = tls.open_ssl_context(None)?;
    let conn = Esp32Stream::TLSStream(Box::new(conn));
    let executor = Esp32Executor::new();

    let (h2, conn) = block_on(executor.run(async { handshake(conn).await }))?;
    let task = executor.spawn(async move {
        conn.await.unwrap();
    });

    let mut robot_client = RobotClient::new(executor, h2, task, config);

    robot_client.request_jwt_token()?;
    robot_client.read_config()?;
    if config.main_handle.is_none() {
        loop {
            if let Some(_r) = wait_notification(Some(Duration::from_secs(30))) {
                log::info!("connection incomming the client task will stop");
                break;
            }
        }
    }
    log::error!("current task handle {:?}", unsafe {
        xTaskGetCurrentTaskHandle()
    });
    Ok(())
}

/// C compatible entry function
extern "C" fn client_entry(config: *mut c_void) {
    let config: Box<RobotClientConfig> = unsafe { Box::from_raw(config as *mut RobotClientConfig) };
    if let Some(err) = clientloop(&config).err() {
        log::error!("client returned with error {}", err);
    }
    if let Some(hnd) = config.main_handle {
        unsafe {
            let _ = notify(hnd, 0);
        }
    }
    unsafe {
        vTaskDelete(std::ptr::null_mut());
    }
}
