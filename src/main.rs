use std::sync::Arc;

use clap::{Parser, ValueEnum};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use truco_domain_engine::{
    juego::{Con, Truco, TrucoBuilder},
    maquina_de_estados::{Cero, Cinco, Cuatro, Dos, Seis, Tres, Uno},
};

#[derive(ValueEnum, Clone, Copy)]
enum NumeroJugadores {
    Dos = 2,
    Cuatro = 4,
    Seis = 6,
}

#[derive(Parser)]
#[command(author, version, about, long_about = "Sever for hosting a Truco game")]
struct Cli {
    #[arg(short, long, default_value_t = 1234)]
    port: u16,
    #[arg(short, long, default_value_t = 30)]
    hasta: u8,
    #[arg(short, long, value_enum, default_value_t = NumeroJugadores::Dos)]
    jugadores: NumeroJugadores,
}

enum BuilderCount {
    Cero(TrucoBuilder<Con, Cero>),
    Uno(TrucoBuilder<Con, Uno>),
    Dos(TrucoBuilder<Con, Dos>),
    Tres(TrucoBuilder<Con, Tres>),
    Cuatro(TrucoBuilder<Con, Cuatro>),
    Cinco(TrucoBuilder<Con, Cinco>),
    Seis(TrucoBuilder<Con, Seis>),
}

impl BuilderCount {
    pub fn add_player(self, str: &str) -> Self {
        match self {
            BuilderCount::Cero(truco_builder) => {
                BuilderCount::Uno(truco_builder.add_player(str.to_string()))
            }
            BuilderCount::Uno(truco_builder) => {
                BuilderCount::Dos(truco_builder.add_player(str.to_string()))
            }
            BuilderCount::Dos(truco_builder) => {
                BuilderCount::Tres(truco_builder.add_player(str.to_string()))
            }
            BuilderCount::Tres(truco_builder) => {
                BuilderCount::Cuatro(truco_builder.add_player(str.to_string()))
            }
            BuilderCount::Cuatro(truco_builder) => {
                BuilderCount::Cinco(truco_builder.add_player(str.to_string()))
            }
            BuilderCount::Cinco(truco_builder) => {
                BuilderCount::Seis(truco_builder.add_player(str.to_string()))
            }
            BuilderCount::Seis(_) => panic!("To many players"),
        }
    }

    pub fn build(self) -> Truco {
        match self {
            BuilderCount::Dos(truco_builder) => truco_builder.build(),
            BuilderCount::Cuatro(truco_builder) => truco_builder.build(),
            BuilderCount::Seis(truco_builder) => truco_builder.build(),
            _ => panic!("Unbuildable"),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let port = cli.port.to_string();

    let listener = TcpListener::bind("0.0.0.0:".to_string() + &port)
        .await
        .unwrap();

    println!("Listening on port: {port}");

    let mut players = Vec::with_capacity(cli.jugadores as usize);

    let game = Arc::new(Mutex::new({
        let mut builder = BuilderCount::Cero(TrucoBuilder::new().hasta(cli.hasta));
        for _ in 0..cli.jugadores as u8 {
            let player = get_player(&listener).await;
            builder = builder.add_player(player.name());
            players.push(player);
            let player_count = players.len();
            for p in &mut players {
                p.send(&format!("{}/{}", player_count, cli.jugadores as u8))
                    .await;
            }
        }
        builder.build()
    }));

    for p in players {
        let game_c = Arc::clone(&game);
        tokio::spawn(async move {
            handle_player(p, game_c);
        });
    }
}

fn handle_player(_player: AuthenticadedUser, _game: Arc<Mutex<Truco>>) {
    todo!();
}

async fn get_player(listener: &TcpListener) -> AuthenticadedUser {
    let (socket, addr) = listener.accept().await.unwrap();
    println!("Conection from: {addr:?}");
    AuthenticadedUser::authenticate(socket).await.unwrap()
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
        Ok(AuthenticadedUser { stream, name })
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&mut self, str: &str) {
        self.stream.write_all(str.as_bytes()).await.unwrap();
    }
}
