# Documentação dos Scripts

Este documento descreve detalhadamente a responsabilidade de cada script no repositório.

## Raiz

### `setup.sh`
**Propósito**: Realizar a instalação inicial e configuração do ambiente.
- Cria a estrutura de diretórios em `~/.config/aliases`.
- Detecta o shell em uso (Bash/Zsh).
- Injeta o bloco de inicialização (source) no arquivo `.rc` do shell.
- Usa marcadores (`# >>> aliases-manager >>>`) para garantir idempotência (pode ser executado várias vezes sem duplicar código).

### `uninstall.sh`
**Propósito**: Remover a ferramenta do sistema.
- Faz a limpeza do arquivo `.rc` removendo o bloco de inicialização.
- Pergunta ao usuário se deseja remover os arquivos de configuração.

## `share/`

### `share/init.sh`
**Propósito**: Script carregado a cada nova sessão do shell.
- Adiciona `bin/` ao `$PATH`.
- Carrega todos os arquivos `.alias` de `~/.config/aliases/aliases.d/` em ordem alfabética.
- Define funções do shell que precisam alterar o ambiente atual (`alias-cd`, `alias-reload`, `alias-edit`).

## `lib/`

### `lib/utils.sh`
**Propósito**: Biblioteca de funções e constantes compartilhadas.
- Define cores para output (vermelho, verde, amarelo).
- Define caminhos padrões (`XDG_CONFIG_HOME`).
- Contém funções auxiliares como `print_success`, `print_error`, `ensure_dir`.

## `bin/` (Executáveis)

Estes scripts são adicionados ao PATH e podem ser executados como comandos.

### `alias-create`
**Uso**: `alias-create <nome> <comando> [arquivo]`
- Cria um novo alias.
- Escapa caracteres especiais automaticamente.
- Escreve no arquivo especificado (ou `local.alias` se omitido).
- Verifica duplicatas antes de escrever.

### `alias-remove`
**Uso**: `alias-remove <nome>`
- Remove um alias existente.
- Gera backup automático do arquivo antes da edição (`.bak`).
- Usa arquivos temporários para garantir atomicidade na escrita.

### `alias-list`
**Uso**: `alias-list`
- Lista todos os aliases configurados.
- Mostra: Nome do alias, arquivo de origem e o comando executado.

### `alias-find`
**Uso**: `alias-find <termo>`
- Busca textual em todos os arquivos de aliases.
- Retorna o arquivo e a linha onde o termo foi encontrado.

### `alias-doctor`
**Uso**: `alias-doctor`
- Script de diagnóstico.
- Verifica:
    - Estrutura de diretórios.
    - Path correto.
    - Integração no arquivo `.rc`.
    - Permissões de execução.

### `alias-enable` / `alias-disable`
**Uso**: `alias-enable <nome>` / `alias-disable <nome>`
- Gerencia grupos de aliases renomeando arquivos.
- `disable`: Adiciona extensão `.disabled` ao arquivo `.alias`.
- `enable`: Remove a extensão `.disabled`.
