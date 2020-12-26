use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use warp::{sse::ServerSentEvent, Filter};

static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

pub type Clients = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

#[derive(Debug)]
pub enum Message {
    ClientId(usize),
    Reply(String),
}

#[derive(Debug)]
struct NotUtf8;
impl warp::reject::Reject for NotUtf8 {}

pub fn chat_routes(
    clients: Clients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let prefix = warp::path!("chat" / ..);

    prefix.and(
        index()
            .or(chat_send(clients.clone()))
            .or(chat_recv(clients)),
    )
}

fn chat_send(
    clients: Clients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("chat")
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
        .and(with_clients(clients))
        .map(|my_id, msg, clients| {
            client_message(my_id, msg, &clients);
            warp::reply()
        })
}

fn chat_recv(
    clients: Clients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("events")
        .and(warp::get())
        .and(with_clients(clients))
        .map(|clients| {
            let stream = client_connected(clients);
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        })
}

fn index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end().map(|| {
        warp::http::Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .body(INDEX_HTML)
    })
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn client_connected(
    clients: Clients,
) -> impl Stream<Item = Result<impl ServerSentEvent + Send + 'static, warp::Error>> + Send + 'static
{
    let my_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    log::debug!("New chat client: {}", my_id);

    let (tx, rx) = mpsc::unbounded_channel();

    tx.send(Message::ClientId(my_id)).unwrap();

    clients.lock().unwrap().insert(my_id, tx);

    rx.map(|msg| match msg {
        Message::ClientId(my_id) => {
            Ok((warp::sse::event("message"), warp::sse::data(my_id)).into_a())
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
