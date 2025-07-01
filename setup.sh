#!/bin/bash

# **Aliases Manager Setup Script** 🚀
# Script para configuração automática do gerenciador de aliases

set -e  # Para em caso de erro

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Função para exibir mensagens coloridas
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo -e "${BLUE}🚀 Configurando Aliases Manager...${NC}"
    echo "=============================================="
}

# Detecta o shell atual e define o arquivo de configuração
detect_shell() {
    if [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
        SHELL_NAME="zsh"
    elif [[ "$SHELL" == *"bash"* ]]; then
        SHELL_CONFIG="$HOME/.bashrc"
        SHELL_NAME="bash"
    else
        print_warning "Shell não detectado automaticamente. Usando .bashrc como padrão."
        SHELL_CONFIG="$HOME/.bashrc"
        SHELL_NAME="bash"
    fi
    
    print_info "Shell detectado: $SHELL_NAME"
    print_info "Arquivo de configuração: $SHELL_CONFIG"
}

# Verifica se já existe configuração no arquivo
check_existing_config() {
    local search_pattern="$1"
    local config_file="$2"
    
    if [[ -f "$config_file" ]] && grep -q "$search_pattern" "$config_file"; then
        return 0  # Existe
    else
        return 1  # Não existe
    fi
}

# Configurar carregamento automático de aliases
setup_auto_loading() {
    print_info "Configurando carregamento automático de aliases..."
    
    local alias_loading_code='
# Carrega todos os arquivos de aliases com extensão .alias da pasta ~/aliases
if [ -d ~/aliases ]; then
    # Usando shopt para permitir que o glob retorne vazio quando não há matches
    shopt -s nullglob 2>/dev/null || setopt NULL_GLOB 2>/dev/null
    for alias_file in ~/aliases/src/*.alias; do
        if [ -f "$alias_file" ]; then
            source "$alias_file"
        fi
    done
    # Resetando a opção
    shopt -u nullglob 2>/dev/null || unsetopt NULL_GLOB 2>/dev/null
fi'

    if check_existing_config "# Carrega todos os arquivos de aliases" "$SHELL_CONFIG"; then
        print_warning "Carregamento automático de aliases já configurado em $SHELL_CONFIG"
    else
        echo "$alias_loading_code" >> "$SHELL_CONFIG"
        print_success "Carregamento automático de aliases adicionado a $SHELL_CONFIG"
    fi
}

# Configurar PATH
setup_path() {
    print_info "Configurando PATH para incluir ~/aliases..."
    
    local path_export='export PATH="$HOME/aliases:$PATH"'
    
    if check_existing_config "aliases.*PATH" "$SHELL_CONFIG"; then
        print_warning "PATH já configurado em $SHELL_CONFIG"
    else
        echo "" >> "$SHELL_CONFIG"
        echo "# Adiciona o diretório aliases ao PATH" >> "$SHELL_CONFIG"
        echo "$path_export" >> "$SHELL_CONFIG"
        print_success "PATH configurado em $SHELL_CONFIG"
    fi
}

# Tornar scripts executáveis
make_scripts_executable() {
    print_info "Tornando scripts executáveis..."
    
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local scripts=("alias-create" "alias-remove")
    
    for script in "${scripts[@]}"; do
        local script_path="$script_dir/$script"
        if [[ -f "$script_path" ]]; then
            chmod +x "$script_path"
            print_success "Script $script tornado executável"
        else
            print_error "Script $script não encontrado em $script_path"
        fi
    done
}

# Criar diretório de aliases se não existir
create_aliases_directory() {
    if [[ ! -d "$HOME/aliases" ]]; then
        print_info "Criando link simbólico para o diretório de aliases..."
        local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        ln -sf "$script_dir" "$HOME/aliases/src"
        print_success "Link simbólico criado: $HOME/aliases -> $script_dir"
    else
        print_warning "Diretório ~/aliases já existe"
    fi
}

# Recarregar configuração do shell
reload_shell_config() {
    print_info "Recarregando configuração do shell..."
    
    if [[ "$SHELL_NAME" == "zsh" ]]; then
        if command -v zsh >/dev/null 2>&1; then
            source "$SHELL_CONFIG" 2>/dev/null || true
            print_success "Configuração do zsh recarregada"
        fi
    else
        if command -v bash >/dev/null 2>&1; then
            source "$SHELL_CONFIG" 2>/dev/null || true
            print_success "Configuração do bash recarregada"
        fi
    fi
}

# Exibir instruções finais
show_final_instructions() {
    echo ""
    echo "=============================================="
    print_success "Setup concluído com sucesso! 🎉"
    echo ""
    print_info "Próximos passos:"
    echo "1. Reinicie seu terminal ou execute: source $SHELL_CONFIG"
    echo "2. Teste criando um alias: alias-create teste 'echo Hello World' exemplo"
    echo "3. Teste removendo o alias: alias-remove teste exemplo"
    echo ""
    print_info "Para mais informações, consulte o README.md"
    echo "=============================================="
}

# Função principal
main() {
    print_header
    
    # Verificar se estamos no diretório correto
    if [[ ! -f "alias-create" ]] || [[ ! -f "alias-remove" ]]; then
        print_error "Scripts alias-create ou alias-remove não encontrados!"
        print_error "Certifique-se de executar este script na raiz do projeto aliases."
        exit 1
    fi
    
    detect_shell
    create_aliases_directory
    make_scripts_executable
    setup_auto_loading
    setup_path
    reload_shell_config
    show_final_instructions
}

# Executar apenas se o script for chamado diretamente
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi 