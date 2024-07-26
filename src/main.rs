use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let port = "1234";

    let listener = TcpListener::bind("0.0.0.0:".to_string() + port)
        .await
        .unwrap();

    println!("Listening on port: {port}");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("Conection from: {addr:?}");
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}

async fn handle_client(socket: TcpStream) {
    let mut authenticaded_user = AuthenticadedUser::authenticate(socket).await;
    loop {
        authenticaded_user.lobby().await;
        authenticaded_user.game().await;
    }
}

struct AuthenticadedUser {
    stream: TcpStream,
    name: String,
}

impl AuthenticadedUser {
    async fn authenticate(mut stream: TcpStream) -> Result<AuthenticadedUser, &'static str> {
        let mut buffer = [0; 1024];
        if let Err(_e) = stream.write(b"Enter Name").await {
            return Err("Fallo al escribir");
        };
        let n_bytes = stream.read(&mut buffer).await.unwrap();
        let name = String::from_utf8_lossy(&buffer[0..n_bytes]).to_string();
        AuthenticadedUser { stream, name }
    }

    async fn lobby(&mut self) {
        let _ = self.stream.write(self.name.as_bytes()).await;
    }

    async fn game(&mut self) {
        let _ = self.stream.write(b"").await;
    }
}
