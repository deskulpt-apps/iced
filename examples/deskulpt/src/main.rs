use std::sync::OnceLock;

use iced::futures::channel::mpsc;
use iced::futures::{FutureExt, SinkExt, StreamExt};
use iced::widget::{button, column, text};
use iced::{Center, Element, Subscription};
use iced_winit::winit;
use serde::{Deserialize, Serialize};
use wry::http;

pub fn main() -> iced::Result {
    let (cmd_tx, cmd_rx) = mpsc::unbounded();

    iced::application(move || App::new(cmd_tx.clone()), App::update, App::view)
        .subscription(App::subscription)
        .run_with_hooks(AppHooks::new(cmd_rx))
}

static IPC_TX: OnceLock<mpsc::UnboundedSender<String>> = OnceLock::new();

struct App {
    value: i64,
    cmd_tx: mpsc::UnboundedSender<WebCmd>,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    Web(WebEvent),
}

#[derive(Debug, Clone)]
enum WebEvent {
    Ready(mpsc::UnboundedSender<String>),
    Ipc(String),
}

#[derive(Debug)]
enum WebCmd {
    EvalJs(String),
}

#[derive(Debug, Deserialize)]
struct IpcRequest {
    id: u64,
    cmd: String,
    #[serde(default)]
    args: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct IpcResponse {
    id: u64,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl App {
    fn new(cmd_tx: mpsc::UnboundedSender<WebCmd>) -> Self {
        Self { value: 0, cmd_tx }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
            Message::Web(event) => match event {
                WebEvent::Ready(sender) => {
                    if let Err(e) = IPC_TX.set(sender) {
                        eprintln!("Failed to set IPC sender: {e:?}");
                    }
                }
                WebEvent::Ipc(raw) => {
                    let request: IpcRequest = match serde_json::from_str(&raw) {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("Failed to parse IPC request: {e}");
                            return;
                        }
                    };
                    match request.cmd.as_str() {
                        "greet" => {
                            let name = request
                                .args
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("world");
                            self.send_ipc_response(IpcResponse {
                                id: request.id,
                                ok: true,
                                data: Some(serde_json::json!({
                                  "message": format!("Hello, {name}!"),
                                    "counter": self.value,
                                })),
                                error: None,
                            });
                        }
                        "add" => {
                            let a = request
                                .args
                                .get("a")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.);
                            let b = request
                                .args
                                .get("b")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.);
                            self.send_ipc_response(IpcResponse {
                                id: request.id,
                                ok: true,
                                data: Some(serde_json::json!(a + b)),
                                error: None,
                            });
                        }
                        "time_ms" => {
                            let ms = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);
                            self.send_ipc_response(IpcResponse {
                                id: request.id,
                                ok: true,
                                data: Some(serde_json::json!(ms)),
                                error: None,
                            });
                        }
                        other => {
                            self.send_ipc_response(IpcResponse {
                                id: request.id,
                                ok: false,
                                data: None,
                                error: Some(format!(
                                    "Unknown command: {other}"
                                )),
                            });
                        }
                    }
                }
            },
        }
    }

    fn send_ipc_response(&self, response: IpcResponse) {
        let payload = match serde_json::to_string(&response) {
            Ok(payload) => payload,
            Err(e) => {
                eprintln!("Failed to serialize IPC response: {e}");
                return;
            }
        };

        let js = format!(
            "window.__demo_ipc && window.__demo_ipc._handleResponse({payload});"
        );
        if let Err(e) = self.cmd_tx.unbounded_send(WebCmd::EvalJs(js)) {
            eprintln!("Failed to send IPC response command to webview: {e}");
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .align_x(Center)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            iced::stream::channel(
                100,
                |mut output: mpsc::Sender<Message>| async move {
                    let (tx, mut rx) = mpsc::unbounded();
                    if let Err(e) =
                        output.send(Message::Web(WebEvent::Ready(tx))).await
                    {
                        eprintln!("Failed to send ready message: {e}");
                    }
                    while let Some(raw) = rx.next().await {
                        if let Err(e) =
                            output.send(Message::Web(WebEvent::Ipc(raw))).await
                        {
                            eprintln!("Failed to send IPC message: {e}");
                        }
                    }
                    println!("rx is going out of scope, ending subscription");
                },
            )
        })
    }
}

#[derive(rust_embed::Embed)]
#[folder = "assets/"]
struct Assets;

struct WebViewWindow {
    webview: wry::WebView,
    #[cfg(target_os = "linux")]
    window: gtk::Window,
    #[cfg(not(target_os = "linux"))]
    window: winit::window::Window,
}

struct AppHooks {
    webview_window: Option<WebViewWindow>,
    cmd_rx: mpsc::UnboundedReceiver<WebCmd>,
}

impl AppHooks {
    fn new(cmd_rx: mpsc::UnboundedReceiver<WebCmd>) -> Self {
        Self {
            webview_window: None,
            cmd_rx,
        }
    }
}

impl iced_winit::EventLoopHooks for AppHooks {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.webview_window.is_some() {
            return;
        }

        let builder = wry::WebViewBuilder::new()
            .with_custom_protocol("deskulpt".to_string(), |_, request| {
                let mut path =
                    request.uri().path().trim_start_matches('/').to_string();
                println!("Requesting path: {}", path);
                if path.is_empty() {
                    path = "index.html".to_string();
                }

                if path.contains("..") || path.contains("\\") {
                    return http::Response::builder()
                        .status(http::StatusCode::BAD_REQUEST)
                        .header(
                            http::header::CONTENT_TYPE,
                            "text/plain; charset=utf-8",
                        )
                        .body(b"Bad path".into())
                        .expect("Failed to build HTTP response");
                }

                let Some(file) = Assets::get(&path) else {
                    return http::Response::builder()
                        .status(http::StatusCode::NOT_FOUND)
                        .header(
                            http::header::CONTENT_TYPE,
                            "text/plain; charset=utf-8",
                        )
                        .body(b"Not found".into())
                        .expect("Failed to build HTTP response");
                };

                let mime_type =
                    mime_guess::from_path(&path).first_or_octet_stream();
                http::Response::builder()
                    .status(http::StatusCode::OK)
                    .header(http::header::CONTENT_TYPE, mime_type.as_ref())
                    .body(file.data)
                    .expect("Failed to build HTTP response")
            })
            .with_ipc_handler(|request| {
                println!("Received IPC message: {:?}", request);
                if let Some(tx) = IPC_TX.get() {
                    if let Err(e) = tx.unbounded_send(request.body().clone()) {
                        eprintln!("Failed to send IPC message to app: {e}");
                    }
                }
            });

        #[cfg(target_os = "windows")]
        let builder = builder.with_url("http://deskulpt.localhost");
        #[cfg(not(target_os = "windows"))]
        let builder = builder.with_url("deskulpt://localhost");

        #[cfg(target_os = "linux")]
        {
            gtk::init().expect("Failed to initialize GTK");

            let window = gtk::Window::new(gtk::WindowType::Toplevel);
            window.set_title("Deskulpt Canvas");
            window.maximize();

            let fixed = gtk::Fixed::new();
            window.add(&fixed);

            let webview =
                builder.build_gtk(&fixed).expect("Failed to create webview");

            window.show_all();
            self.webview_window = Some(WebViewWindow { webview, window });
        }

        #[cfg(not(target_os = "linux"))]
        {
            use winit::window::WindowAttributes;

            let attributes = WindowAttributes::default();

            let window = event_loop
                .create_window(attributes)
                .expect("Failed to create window");

            let webview =
                builder.build(&window).expect("Failed to create webview");

            self.webview_window = Some(WebViewWindow { webview, window });
        }
    }

    fn about_to_wait(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        if let Some(w) = &self.webview_window {
            while let Some(cmd) = self.cmd_rx.next().now_or_never().flatten() {
                match cmd {
                    WebCmd::EvalJs(js) => {
                        if let Err(e) = w.webview.evaluate_script(&js) {
                            eprintln!("Failed to evaluate script: {e}");
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }
}
