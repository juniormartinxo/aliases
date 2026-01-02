# Gerenciador de Aliases

Uma ferramenta CLI robusta e agnóstica de shell para gerenciar seus aliases, seguindo os padrões XDG.

## Funcionalidades

- **Armazenamento Padronizado**: Armazena aliases em `~/.config/aliases/aliases.d/`.
- **Modular**: Organiza aliases em arquivos separados (ex: `10-git.alias`, `20-docker.alias`).
- **Setup Idempotente**: É seguro rodar o `setup.sh` múltiplas vezes.
- **Diagnóstico**: Comando `alias-doctor` embutido.
- **Buscável**: Encontre aliases facilmente com `alias-find`.

## Instalação

```bash
git clone https://github.com/seuusuario/aliases.git
cd aliases
./setup.sh
```

Reinicie seu terminal ou execute `source ~/.zshrc` (ou `~/.bashrc`) para aplicar.

## Uso

### Comandos

| Comando | Descrição |
|---------|-------------|
| `alias-create <nome> <cmd> [arquivo]` | Cria um novo alias. Padrão: `local.alias`. |
| `alias-remove <nome>` | Remove um alias. |
| `alias-list` | Lista todos os aliases gerenciados. |
| `alias-find <termo>` | Busca por aliases. |
| `alias-edit <nome>` | Edita o arquivo contendo o alias (requer `$EDITOR`). |
| `alias-reload` | Recarrega aliases imediatamente. |
| `alias-enable <nome>` | Habilita um arquivo de alias desativado. |
| `alias-disable <nome>` | Desativa um arquivo de alias. |
| `alias-doctor` | Verifica a integridade da instalação. |

### Exemplos

```bash
# Criar um novo alias na categoria 'git'
alias-create gs 'git status' git

# Listar todos os aliases
alias-list

# Buscar aliases relacionados ao docker
alias-find docker
```

## Estrutura

```
~/.config/aliases/
  aliases.d/
    00-core.alias    # Aliases principais
    local.alias      # Aliases locais/customizados
    ...
```

Arquivos são carregados em ordem alfabética. `local.alias` é sempre carregado por último.

## Desenvolvimento

- **Testes**: Execute `bats tests/`
- **Linting**: Execute `shellcheck bin/*`

## Desinstalação

Execute `./uninstall.sh` para remover a integração com o shell. Você será perguntado se deseja remover o diretório de configuração.
