use futures::{Stream, StreamExt};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use tokio::sync::mpsc;
use warp::{sse::ServerSentEvent, Filter};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let clients = Arc::new(Mutex::new(HashMap::new()));
    let clients = warp::any().map(move || clients.clone());

    let chat_send = warp::path("chat")
        .and(warp::post())
        .and(warp::path::param::<usize>())
        .and(warp::body::content_length_limit(500))
        .and(
            warp::body::bytes().and_then(|body: bytes::Bytes| async move {
                std::str::from_utf8(&body)
                    .map(String::from)
                    .map_err(|_e| warp::reject::custom(NotUtf8))
            }),
        )
        .and(clients.clone())
        .map(|my_id, msg, clients| {
            client_message(my_id, msg, &clients);
            warp::reply()
        });

    let chat_recv = warp::path("chat")
        .and(warp::get())
        .and(clients)
        .map(|clients| {
            let stream = client_connected(clients);
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });

    let index = warp::path::end().map(|| {
        warp::http::Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .body(INDEX_HTML)
    });

    let routes = index.or(chat_recv).or(chat_send);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
enum Message {
    ClientId(usize),
    Reply(String),
}

#[derive(Debug)]
struct NotUtf8;
impl warp::reject::Reject for NotUtf8 {}

type Clients = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

fn client_connected(
    clients: Clients,
) -> impl Stream<Item = Result<impl ServerSentEvent + Send + 'static, warp::Error>> + Send + 'static
{
    let my_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("New chat client: {}", my_id);

    let (tx, rx) = mpsc::unbounded_channel();

    tx.send(Message::ClientId(my_id)).unwrap();

    clients.lock().unwrap().insert(my_id, tx);

    rx.map(|msg| match msg {
        Message::ClientId(my_id) => {
            Ok((warp::sse::event("client"), warp::sse::data(my_id)).into_a())
        }
        Message::Reply(reply) => Ok(warp::sse::data(reply).into_b()),
    })
}

fn client_message(my_id: usize, msg: String, clients: &Clients) {
    let new_msg = format!("<User#{}>: {}", my_id, msg);

    clients.lock().unwrap().retain(|uid, tx| {
        if my_id == *uid {
            true // don't send, but do retain
        } else {
            tx.send(Message::Reply(new_msg.clone())).is_ok() // send message and retain if successful
        }
    });
}

static INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
    <head>
        <title>Warp Chat</title>
    </head>
    <body>
        <h1>warp chat</h1>
        <div id="chat">
            <p><em>Connecting...</em></p>
        </div>
        <input type="text" id="text" />
        <button type="button" id="send">Send</button>
        <script type="text/javascript">
        var uri = 'http://' + location.host + '/chat';
        var sse = new EventSource(uri);
        function message(data) {
            var line = document.createElement('p');
            line.innerText = data;
            chat.appendChild(line);
        }
        sse.onopen = function() {
            chat.innerHTML = "<p><em>Connected!</em></p>";
        }
        var user_id;
        sse.addEventListener("client", function(msg) {
            user_id = msg.data;
        });
        sse.onmessage = function(msg) {
            message(msg.data);
        };
        send.onclick = function() {
            var msg = text.value;
            var xhr = new XMLHttpRequest();
            xhr.open("POST", uri + '/' + user_id, true);
            xhr.send(msg);
            text.value = '';
            message('<You>: ' + msg);
        };
        </script>
    </body>
</html>
"#;
