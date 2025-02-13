use std::{env, fs::File, io::{self, BufRead}};
use log::{error, info, debug};
use thiserror::Error;
use tokio::{time::sleep, process::Command};
use reqwest::Client;
use std::time::Duration;

#[derive(Error, Debug)]
enum AppError {
    #[error("Erro ao executar comando: {0}")]
    CommandError(#[from] std::io::Error),
    #[error("Erro na requisição HTTP: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Erro de configuração: {0}")]
    ConfigError(String),
}

struct Config {
    url: String,
    interval: u64,
    timeout: u64,
}

impl Config {
    fn from_env() -> Result<Self, AppError> {
        dotenv::dotenv().ok();

        let url = env::var("API_URL")
            .map_err(|_| AppError::ConfigError("API_URL não especificada no .env".to_string()))?;

        let interval = env::var("CHECK_INTERVAL")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| AppError::ConfigError("CHECK_INTERVAL inválido".to_string()))?;

        let timeout = env::var("REQUEST_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string()) // Timeout padrão: 30s
            .parse()
            .map_err(|_| AppError::ConfigError("REQUEST_TIMEOUT inválido".to_string()))?;

        Ok(Config { url, interval, timeout })
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    match Config::from_env() {
        Ok(config) => {
            info!("Iniciando monitoramento com URL configurada");
            start_loop(config).await;
        }
        Err(e) => {
            error!("Erro ao iniciar: {}", e);
            std::process::exit(1);
        }
    }
}

async fn start_loop(config: Config) {
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .build()
        .expect("Falha ao criar cliente HTTP");

    loop {
        match get_users().await {
            Ok(user_list) => {
                if let Err(e) = send_post_request(&client, config.url.clone(), user_list).await {
                    error!("Erro ao enviar POST request: {}", e);
                }
            }
            Err(e) => {
                error!("Erro ao obter a lista de usuários: {}", e);
            }
        }

        sleep(Duration::from_secs(config.interval)).await;
    }
}

async fn get_users() -> Result<String, AppError> {
    let mut user_list = Vec::new();

    // Executa o comando assíncrono para obter usuários ativos no SSH
    let output = Command::new("sh")
        .arg("-c")
        .arg("ps aux | grep priv | grep Ss | awk -F 'sshd: ' '{print $2}' | awk -F ' ' '{print $1}'")
        .output()
        .await?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        user_list.push(line.to_string());
    }

    // Verifica usuários OpenVPN
    if let Ok(users) = get_openvpn_users().await {
        user_list.extend(users);
    }

    Ok(user_list.join(","))
}

async fn get_openvpn_users() -> Result<Vec<String>, AppError> {
    let path = "/etc/openvpn/openvpn-status.log";
    if !tokio::fs::metadata(path).await.is_ok() {
        return Ok(Vec::new());
    }

    let file = tokio::fs::File::open(path).await?;
    let reader = io::BufReader::new(file);
    let mut users = Vec::new();

    let mut lines = reader.lines();
    while let Some(line) = lines.next().transpose()? {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() > 1 && parts[1].contains('.') {
            users.push(parts[0].to_string());
        }
    }

    Ok(users)
}

async fn send_post_request(client: &Client, url: String, user_list: String) -> Result<(), AppError> {
    let form_data = format!("users={}", urlencoding::encode(&user_list));
    debug!("Enviando dados para a URL configurada");

    client.post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_data)
        .send()
        .await?;

    info!("Usuários enviados com sucesso");
    Ok(())
}