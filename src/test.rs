use super::db;
use super::model::Message;
use super::rocket;
use super::web;
use rocket::http::Status;
use rocket::local::Client;

// For tests to run asynchronously, each one of them must have a different stream_id

#[test]
fn returns_empty_streams() {
    let stream_id = [0, 0, 0, 0, 0, 0, 0, 0];
    let (client, _) = setup_with_stream(&stream_id);
    let mut response = client.get("/streams/AAAAAAAAAAA=/messages").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some("{\"id\":\"AAAAAAAAAAA=\",\"messages\":[]}".into())
    );
}

#[test]
fn returns_few_messages() {
    let stream_id = [0, 0, 0, 0, 0, 0, 0, 1];
    let (client, db_conn) = setup_with_stream(&stream_id);

    db::add_message(&db_conn, &stream_id, message(1)).unwrap();
    db::add_message(&db_conn, &stream_id, message(2)).unwrap();

    let mut response = client.get("/streams/AAAAAAAAAAE=/messages").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some(
        format!(
            "{{\"id\":\"AAAAAAAAAAE=\",\"messages\":[{{\"signature\":\"{}\",\"data\":\"{}\"}},{{\"signature\":\"{}\",\"data\":\"{}\"}}]}}",
            encode64(&message(1).signature),encode64(&message(1).data),
            encode64(&message(2).signature),encode64(&message(2).data),
    ).into()));
}

#[test]
fn returns_filtered_messages() {
    let stream_id = [0, 0, 0, 0, 0, 0, 0, 2];
    let (client, db_conn) = setup_with_stream(&stream_id);

    db::add_message(&db_conn, &stream_id, message(1)).unwrap();
    db::add_message(&db_conn, &stream_id, message(2)).unwrap();
    db::add_message(&db_conn, &stream_id, message(3)).unwrap();

    let mut response = client
        .get("/streams/AAAAAAAAAAI=/messages?offset=1&limit=1")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some(
            format!(
            "{{\"id\":\"AAAAAAAAAAI=\",\"messages\":[{{\"signature\":\"{}\",\"data\":\"{}\"}}]}}",
            encode64(&message(2).signature),encode64(&message(2).data),
    )
            .into()
        )
    );
}

#[test]
fn adds_message_to_stream() {
    let stream_id = [0, 0, 0, 0, 0, 0, 0, 3];
    let (client, _) = setup_with_stream(&stream_id);

    let response = client
        .post("/streams/AAAAAAAAAAM=/messages")
        .body(String::from(format!(
            "{{\"signature\":\"{}\",\"data\":\"{}\"}}",
            encode64(&message(1).signature),
            encode64(&message(1).data),
        )))
        .header(rocket::http::ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let mut response = client.get("/streams/AAAAAAAAAAM=/messages").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.body_string(),
        Some(
            format!(
            "{{\"id\":\"AAAAAAAAAAM=\",\"messages\":[{{\"signature\":\"{}\",\"data\":\"{}\"}}]}}",
            encode64(&message(1).signature),encode64(&message(1).data),
    )
            .into()
        )
    );
}

#[test]
fn returns_404_in_json() {
    let client = setup_client();
    let response = client.get("/non_existent_route").dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.headers().get_one("Content-Type"), Some("application/json"));
}

#[test]
fn returns_404_if_invalid_id() {
    let client = setup_client();
    let response = client.get("/streams/AAAAAAAAAA=/messages").dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.headers().get_one("Content-Type"), Some("application/json"));
}

#[test]
fn returns_404_if_invalid_id_on_post() {
    let client = setup_client();
    let response = client
        .post("/streams/AAAAAAAAAA=/messages")
        .body(String::from(format!(
            "{{\"signature\":\"{}\",\"data\":\"{}\"}}",
            encode64(&message(1).signature),
            encode64(&message(1).data),
        )))
        .header(rocket::http::ContentType::JSON)
        .dispatch();
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.headers().get_one("Content-Type"), Some("application/json"));
}

#[test]
fn validates_message_signature() {
    let client = setup_client();

    let response = client
        .post("/streams/AAAAAAAAAAM=/messages")
        .body(String::from(format!(
            "{{\"signature\":\"A{}\",\"data\":\"{}\"}}",
            encode64(&message(1).signature),
            encode64(&message(1).data),
        )))
        .header(rocket::http::ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(response.headers().get_one("Content-Type"), Some("application/json"));
}

#[test]
fn validates_message_data() {
    let client = setup_client();

    let response = client
        .post("/streams/AAAAAAAAAAM=/messages")
        .body(String::from(format!(
            "{{\"signature\":\"{}\",\"data\":\"A{}\"}}",
            encode64(&message(1).signature),
            encode64(&message(1).data),
        )))
        .header(rocket::http::ContentType::JSON)
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(response.headers().get_one("Content-Type"), Some("application/json"));
}

fn encode64(bytes: &[u8]) -> String {
    base64::encode_config(bytes, base64::URL_SAFE)
}

fn message(byte: u8) -> Message {
    Message {
        data: [byte; 128],
        signature: [byte; 64],
    }
}

fn setup_with_stream(stream_id: &[u8; 8]) -> (rocket::local::Client, redis::Connection) {
    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let conn = redis_client.get_connection().unwrap();
    let _: () = redis::cmd("DEL").arg(stream_id).query(&conn).unwrap();

    let client = setup_client();

    (client, conn)
}

fn setup_client() -> rocket::local::Client {
    Client::new(web::rocket()).expect("valid rocket instance")
}