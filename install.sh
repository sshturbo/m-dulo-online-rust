#!/bin/bash

# ===============================
# Configurações e Variáveis Globais
# ===============================
APP_DIR="/opt/modulo-online-rust"
DEPENDENCIES=("unzip" "build-essential" "pkg-config" "libssl-dev" "git")
VERSION="1.0.0"
BUILD_DIR="/tmp/modulo-online-rust-build"
SERVICE_FILE_NAME="modulo-online-rust.service"

# ===============================
# Funções Utilitárias
# ===============================
print_centered() {
    printf "\e[1;33m%s\e[0m\n" "$1"
}

print_big_message() {
    echo -e "\e[1;32m"
    echo "========================================"
    echo "       BEM-VINDO AO INSTALADOR"
    echo "        DO MÓDULO ONLINE RUST"
    echo "========================================"
    echo -e "\e[0m"
}

progress_bar() {
    local total_steps=$1
    for ((i = 0; i < total_steps; i++)); do
        echo -n "#"
        sleep 0.1
    done
    echo " COMPLETO!"
}

run_with_spinner() {
    local command="$1"
    local message="$2"
    echo -ne "\e[1;34m$message...\e[0m"
    $command &>/tmp/command_output.log &
    local pid=$!
    while kill -0 $pid 2>/dev/null; do
        echo -n "."
        sleep 1
    done
    wait $pid
    if [ $? -ne 0 ]; then
        echo -e "\e[1;31m ERRO!\e[0m"
        cat /tmp/command_output.log
        exit 1
    else
        echo -e "\e[1;32m FEITO!\e[0m"
    fi
}

install_if_missing() {
    local package=$1
    if ! command -v $package &>/dev/null; then
        run_with_spinner "apt-get install -y $package" "INSTALANDO $package"
    else
        print_centered "$package JÁ ESTÁ INSTALADO."
    fi
}

# ===============================
# Exibir Mensagem Inicial
# ===============================
clear
print_big_message
echo -e "\e[1;36mAntes de iniciar a instalação, precisamos de algumas informações.\e[0m"
echo -e "\e[1;33mPor favor, informe o domínio onde a API será acessada.\e[0m"
echo ""
read -p "Digite o domínio da API (exemplo: api.seusite.com): " API_DOMAIN
API_URL="https://${API_DOMAIN}/online.php"

# ===============================
# Validações Iniciais
# ===============================
if [[ $EUID -ne 0 ]]; then
    echo -e "\e[1;31mEste script deve ser executado como root.\e[0m"
    exit 1
fi

# ===============================
# Atualização do Sistema
# ===============================
print_centered "ATUALIZANDO O SISTEMA..."
run_with_spinner "apt-get update" "ATUALIZANDO O SISTEMA"
run_with_spinner "apt-get upgrade -y" "ATUALIZANDO O SISTEMA"

# Instalar dependências
for dep in "${DEPENDENCIES[@]}"; do
    install_if_missing $dep
    wait $!
done

# ===============================
# Instalação do Rust
# ===============================
print_centered "INSTALANDO RUST..."
if ! command -v rustc &>/dev/null; then
    curl -o /tmp/rustup-init.sh https://sh.rustup.rs
    if [ $? -ne 0 ]; then
        echo -e "\e[1;31mErro ao baixar o instalador do Rust\e[0m"
        exit 1
    fi

    chmod +x /tmp/rustup-init.sh
    run_with_spinner "/tmp/rustup-init.sh -y" "INSTALANDO RUST"
    rm /tmp/rustup-init.sh
    source "$HOME/.cargo/env"
else
    print_centered "RUST JÁ ESTÁ INSTALADO."
fi

# ===============================
# Download e Build do Projeto
# ===============================
print_centered "CLONANDO E COMPILANDO O PROJETO..."
if [ -d "$BUILD_DIR" ]; then
    rm -rf "$BUILD_DIR"
fi
mkdir -p "$BUILD_DIR"

run_with_spinner "git clone https://github.com/sshturbo/m-dulo-online-rust.git $BUILD_DIR" "CLONANDO REPOSITÓRIO"
cd "$BUILD_DIR"

print_centered "COMPILANDO O PROJETO..."
run_with_spinner "cargo build --release" "COMPILANDO"

# ===============================
# Configuração da Aplicação
# ===============================
if [ -d "$APP_DIR" ]; then
    print_centered "DIRETÓRIO $APP_DIR JÁ EXISTE. EXCLUINDO ANTIGO..."
    if systemctl list-units --full -all | grep -Fq "$SERVICE_FILE_NAME"; then
        run_with_spinner "systemctl stop $SERVICE_FILE_NAME" "PARANDO SERVIÇO"
        run_with_spinner "systemctl disable $SERVICE_FILE_NAME" "DESABILITANDO SERVIÇO"
    else
        print_centered "SERVIÇO $SERVICE_FILE_NAME NÃO ENCONTRADO."
    fi
    run_with_spinner "rm -rf $APP_DIR" "EXCLUINDO DIRETÓRIO"
fi

mkdir -p $APP_DIR

# Copiar arquivos necessários
print_centered "INSTALANDO BINÁRIO E ARQUIVOS DE CONFIGURAÇÃO..."
run_with_spinner "cp $BUILD_DIR/target/release/modulo-online-rust $APP_DIR/" "COPIANDO BINÁRIO"
run_with_spinner "cp $BUILD_DIR/.env.exemple $APP_DIR/.env" "COPIANDO ARQUIVO .ENV"
run_with_spinner "cp $BUILD_DIR/$SERVICE_FILE_NAME $APP_DIR/" "COPIANDO ARQUIVO DE SERVIÇO"

# Atualizar API_URL no arquivo .env antes da execução
sed -i "s|API_URL=.*|API_URL=${API_URL}|g" "$APP_DIR/.env"
chmod -R 775 $APP_DIR

# Limpar diretório de build
run_with_spinner "rm -rf $BUILD_DIR" "LIMPANDO ARQUIVOS TEMPORÁRIOS"

# Configurar serviço systemd
if [ -f "$APP_DIR/$SERVICE_FILE_NAME" ]; then
    cp "$APP_DIR/$SERVICE_FILE_NAME" /etc/systemd/system/
    chmod 644 /etc/systemd/system/$SERVICE_FILE_NAME
    systemctl daemon-reload
    systemctl enable $SERVICE_FILE_NAME
    systemctl start $SERVICE_FILE_NAME
    print_centered "SERVIÇO $SERVICE_FILE_NAME CONFIGURADO E INICIADO COM SUCESSO!"
else
    print_centered "Erro: Arquivo de serviço não encontrado."
    exit 1
fi

progress_bar 10
print_big_message
echo -e "\e[1;32mMÓDULO INSTALADO E CONFIGURADO COM SUCESSO!\e[0m"
