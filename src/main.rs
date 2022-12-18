use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use axum::{Router, routing::post};
use axum::extract::State;
use axum::http::{StatusCode};
use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::spawn;
use tokio::sync::Mutex;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tracing::{debug, error, info};

/// Daemon to expose LED controller via HTTP API.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the serial link to use to communicate with Arduino (eg. /dev/ttyUSB0)
    #[arg(short, long)]
    device: String,

    /// Hostname (IP address) to bind the HTTP listener to
    #[arg(long, default_value = "0.0.0.0")]
    hostname: String,

    /// Port to bind HTTP listener to
    #[arg(short, long, default_value_t = 80)]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting LED CTRL daemon...");

    let args = Args::parse();
    let port = match tokio_serial::new(&args.device, 38400).open_native_async() {
        Ok(t) => t,
        Err(e) => {
            error!("Cannot open serial device {}!{:?}", args.device, e);
            return;
        }
    };
    let port = Arc::new(Mutex::new(port));

    // split serial port communication into rx and tx
    spawn(read_serial_link(port.clone()));

    info!("Starting HTTP listener on {}:{}...", args.hostname, args.port);
    create_http_server(port, &args.hostname, args.port).await;
}

async fn read_serial_link(file: Arc<Mutex<SerialStream>>) {
    debug!("Reading serial link...");
    let mut buf = [0u8; 32];

    // read bytes and throw them away
    loop {
        {
            let mut file = file.lock().await;
            match file.read(&mut buf).await {
                Ok(t) => debug!("Read from serial: {}", String::from_utf8_lossy(&buf.as_slice()[0 .. t])),
                Err(e) => error!("Error while reading serial link! {:?}", e),
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

type AppState = Arc<Mutex<SerialStream>>;

async fn create_http_server(file: AppState, hostname: &str, port: u16) {
    let ip_addr = match IpAddr::from_str(hostname) {
        Ok(t) => t,
        Err(e) => {
            error!("Cannot parse provided hostname! {:?}", e);
            return;
        }
    };
    let socket_addr = SocketAddr::from((ip_addr, port));
    let app = Router::new()
        // managed commands
        .route("/on", post(turn_on))
        .route("/off", post(turn_off))
        .route("/intensity_plus", post(intensity_plus))
        .route("/intensity_minus", post(intensity_minus))
        .route("/white", post(white))
        .route("/red", post(red))
        .route("/green", post(green))
        .route("/blue", post(blue))
        // raw commands
        .route("/raw/on", post(uturn_on))
        .route("/raw/off", post(uturn_off))
        .route("/raw/intensity_plus", post(uintensity_plus))
        .route("/raw/intensity_minus", post(uintensity_minus))
        .route("/raw/white", post(uwhite))
        .route("/raw/red", post(ured))
        .route("/raw/green", post(ugreen))
        .route("/raw/blue", post(ublue))
        .with_state(file);

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

macro_rules! cmd {
    ($name: ident, $serial_cmd: expr) => {
        async fn $name(State(file): State<AppState>) -> StatusCode {
            let mut file = file.lock().await;
            match file.write_all($serial_cmd.as_bytes()).await {
                Ok(_) => StatusCode::OK,
                Err(e) => {
                    error!("Cannot write {} command to device! {:?}", $serial_cmd, e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    };
}

cmd!(turn_on, "LED_ON");
cmd!(turn_off, "LED_OFF");
cmd!(intensity_plus, "LED_IP");
cmd!(intensity_minus, "LED_IM");
cmd!(white, "LED_WHITE");
cmd!(red, "LED_RED");
cmd!(green, "LED_GREEN");
cmd!(blue, "LED_BLUE");

// raw commands
cmd!(uturn_on, "ULED_ON");
cmd!(uturn_off, "ULED_OFF");
cmd!(uintensity_plus, "ULED_IP");
cmd!(uintensity_minus, "ULED_IM");
cmd!(uwhite, "ULED_WHITE");
cmd!(ured, "ULED_RED");
cmd!(ugreen, "ULED_GREEN");
cmd!(ublue, "ULED_BLUE");