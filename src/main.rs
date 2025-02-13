use std::{
    env,
    process::Command,
    thread,
    time::Duration,
    fs,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Uso: ./programa <URL>");
        return;
    }
    let url = &args[1];
    start_loop(url);
}

fn start_loop(url: &str) {
    loop {
        match get_users() {
            Ok(user_list) => {
                if let Err(e) = send_post_request(url, &user_list) {
                    println!("Erro ao enviar POST request: {}", e);
                }
            }
            Err(e) => {
                println!("Erro ao obter a lista de usuários: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(3));
    }
}

fn get_users() -> Result<String, Box<dyn std::error::Error>> {
    let mut user_list = Vec::new();

    // Obtém usuários do sistema
    let output = Command::new("sh")
        .arg("-c")
        .arg("ps aux | grep priv | grep Ss")
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.len() >= 12 {
            // Encontra o índice do padrão "sshd:"
            if let Some(index) = columns.iter().position(|&x| x == "sshd:") {
                if index + 1 < columns.len() {
                    user_list.push(columns[index + 1].to_string());
                }
            }
        }
    }

    // Se o arquivo openvpn-status.log existir, adicione os usuários do OpenVPN
    if fs::metadata("/etc/openvpn/openvpn-status.log").is_ok() {
        let output = Command::new("sh")
            .arg("-c")
            .arg("cat /etc/openvpn/openvpn-status.log | grep -Eo '^[a-zA-Z0-9_-]+,[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+' | awk -F, '{print $1}'")
            .output()?;

        let openvpn_users = String::from_utf8_lossy(&output.stdout);
        for user in openvpn_users.lines() {
            if !user.is_empty() {
                user_list.push(user.to_string());
            }
        }
    }

    Ok(user_list.join(","))
}

fn send_post_request(url: &str, user_list: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let form_data = format!("users={}", urlencoding::encode(user_list));
    
    client.post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_data)
        .send()?;

    println!("Enviando lista de usuários {} para {}", user_list, url);
    Ok(())
}
