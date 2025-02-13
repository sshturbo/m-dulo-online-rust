# Módulo Online Rust

Este programa monitora usuários conectados via SSH e OpenVPN, enviando periodicamente a lista de usuários ativos para um servidor remoto.

## Funcionalidades

- Monitoramento de usuários SSH conectados
- Monitoramento de usuários OpenVPN ativos
- Envio automático da lista de usuários via POST request
- Configuração via variáveis de ambiente ou arquivo .env
- Retry automático em caso de falhas
- Logging detalhado das operações

## Configuração

O programa pode ser configurado através de variáveis de ambiente ou arquivo .env:

```env
API_URL=http://exemplo.com/api # URL para envio dos dados
CHECK_INTERVAL=3 # Intervalo entre verificações em segundos
REQUEST_TIMEOUT=5 # Timeout das requisições em segundos
```

## Instalação rapida

```bash
sudo wget -qO- https://raw.githubusercontent.com/sshturbo/m-dulo-online-rust/refs/heads/master/install.sh | sudo bash
```

## Verificar se está instalado e executado com sucesso só executar o comando.
```bash
sudo systemctl status m-dulo.service
```

## Para poder tá parando os módulos e só executar o comando.
```bash
sudo systemctl stop m-dulo.service
```
```bash
sudo systemctl disable m-dulo.service
```
```bash
sudo systemctl daemon-reload
```

## Para poder ta iniciando os módulos e so executar o comando.
```bash
sudo systemctl enable m-dulo.service
```
```bash
sudo systemctl start m-dulo.service
```

## Instalação

Requisitos:
- Rust 1.70 ou superior
- OpenSSL development headers
- Build essentials

```bash
# Instalar dependências (Ubuntu/Debian)
sudo apt-get install build-essential pkg-config libssl-dev

# Compilar o projeto
cargo build --release
```

## Uso

```bash
# Executar com configurações do .env
./target/release/modulo-online-rust

# Ou especificar a URL diretamente
./target/release/modulo-online-rust http://exemplo.com/api

# Para ver logs detalhados
RUST_LOG=info ./target/release/modulo-online-rust
```

## Logs

O programa usa env_logger para logging. Os níveis disponíveis são:
- error: Erros que impedem o funcionamento normal
- info: Informações sobre operações normais
- debug: Informações detalhadas para debugging

## Licença

MIT