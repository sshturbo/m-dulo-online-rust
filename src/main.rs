use std::{
    env,
    process::Command,
    thread,
    time::Duration,
    fs::{self, File},
    io::{self, BufRead},
};
use log::{error, info, debug};
use thiserror::Error;
use backoff::{ExponentialBackoff, Error as BackoffError};

#[derive(Error, Debug)]
enum AppError {
    #[error("Erro ao executar comando: {0}")]
    CommandError(#[from] std::io::Error),
    #[error("Erro na requisição HTTP: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Erro de configuração: {0}")]
    ConfigError(String),
    #[error("Erro de retry: {0}")]
    RetryError(String),
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
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .map_err(|_| AppError::ConfigError("REQUEST_TIMEOUT inválido".to_string()))?;
            
        Ok(Config {
            url,
            interval,
            timeout,
        })
    }
}

fn main() {
    env_logger::init();
    
    match Config::from_env() {
        Ok(config) => {
            info!("Iniciando monitoramento com URL configurada");
            start_loop(&config);
        }
        Err(e) => {
            error!("Erro ao iniciar: {}", e);
            std::process::exit(1);
        }
    }
}

fn start_loop(config: &Config) {
    loop {
        match get_users() {
            Ok(user_list) => {
                if let Err(e) = send_post_request(&config.url, &user_list, config.timeout) {
                    error!("Erro ao enviar POST request: {}", e);
                }
            }
            Err(e) => {
                error!("Erro ao obter a lista de usuários: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(config.interval));
    }
}

fn get_users() -> Result<String, AppError> {
    let mut user_list = Vec::new();

    // Executa o comando para obter os processos relacionados a "priv" em estado "Ss"
    let output = Command::new("sh")
        .arg("-c")
        .arg("ps aux | grep priv | grep Ss | awk -F 'sshd: ' '{print $2}' | awk -F ' ' '{print $1}'")
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        user_list.push(line.to_string());
    }

    // Verifica usuários OpenVPN
    if let Ok(users) = get_openvpn_users() {
        user_list.extend(users);
    }

    Ok(user_list.join(","))
}

fn get_openvpn_users() -> Result<Vec<String>, AppError> {
    let path = "/etc/openvpn/openvpn-status.log";
    if !fs::metadata(path).is_ok() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let users: Vec<String> = reader.lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() > 1 && parts[1].contains('.') {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(users)
}

fn send_post_request(url: &str, user_list: &str, timeout: u64) -> Result<(), AppError> {
    let backoff = ExponentialBackoff::default();
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()?;
    
    let form_data = format!("users={}", urlencoding::encode(user_list));
    debug!("Preparando para enviar dados para a URL configurada");
    
    backoff::retry(backoff, || {
        info!("Fazendo requisição POST");
        let response = client.post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_data.clone())
            .send()
            .map_err(|e| {
                error!("Erro ao enviar requisição: {}", e);
                AppError::from(e)
            })?;
            
        let status = response.status();
        info!("Resposta recebida: Status={}", status);
        
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            error!("Erro na resposta: Status={}", status);
            return Err(backoff::Error::Permanent(AppError::ConfigError(
                format!("Erro HTTP {}: [DADOS SENSÍVEIS]", status)
            )));
        }
        Ok(response)
    })
    .map_err(|e| match e {
        BackoffError::Permanent(e) => {
            error!("Erro permanente no retry: {}", e);
            e
        }
        BackoffError::Transient { err, .. } => {
            error!("Erro transiente no retry: {}", err);
            AppError::RetryError(err.to_string())
        }
    })?;

    info!("Usuários enviados com sucesso");
    Ok(())
}