# Gerenciador de Aliases

Uma ferramenta CLI robusta e agnóstica de shell para gerenciar seus aliases, seguindo os padrões XDG.

## Funcionalidades

- **Armazenamento Padronizado**: Armazena aliases em `~/.config/aliases/aliases.d/`.
- **Modular**: Organiza aliases em arquivos separados (ex: `10-git.alias`, `20-docker.alias`).
- **Setup Idempotente**: É seguro rodar o `setup.sh` múltiplas vezes.
- **Suporte a PowerShell**: Funciona no Windows PowerShell usando o mesmo formato de arquivo de alias.
- **Diagnóstico**: Comando `alias-doctor` embutido.
- **Buscável**: Encontre aliases facilmente com `alias-find`.

## Instalação

### Bash / Zsh

```bash
git clone https://github.com/seuusuario/aliases.git
cd aliases
./setup.sh
```

Reinicie seu terminal ou execute `source ~/.zshrc` (ou `~/.bashrc`) para aplicar.

### PowerShell (Windows)

```powershell
git clone https://github.com/seuusuario/aliases.git
cd aliases
.\setup.ps1
. $PROFILE
```

Isso adiciona a integração ao perfil do PowerShell e disponibiliza comandos como `alias-create`, `alias-list`, `alias-edit` e `alias-reload` nas próximas sessões.
Os aliases gerenciados sao carregados como `Function` do PowerShell. Para inspecionar um alias, use `Get-Command <nome>`. `Get-Alias` mostra apenas aliases nativos do PowerShell.

Se voce executar `.\setup.ps1` novamente, o bloco de integracao em `$PROFILE` sera substituido, sem duplicacao.

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

```powershell
# Criar um alias no PowerShell e recarregar
alias-create gst "git status -sb" git
alias-reload

# Usar o alias normalmente
gst

# Inspecionar um alias gerenciado no PowerShell
Get-Command gst

# Usar sintaxe explicita do PowerShell para executaveis com espaco no caminho
alias-create pfle "& 'C:\Program Files\Notepad++\notepad++.exe' '$PROFILE'" system
pfle

# Executar um script PowerShell
alias-create syncdocs "& 'C:\tools\sync-docs.ps1'" scripts
syncdocs

# Executar um script PowerShell com argumentos fixos
alias-create deploy-api "& 'D:\work\deploy-api.ps1' -Environment dev -Verbose" scripts
deploy-api
```

### Notas para PowerShell

- Os aliases gerenciados sao funcoes do PowerShell, nao entradas `AliasInfo`.
- Use `Get-Command <nome>` para verificar se um alias foi carregado.
- Use `alias-reload` apos criar, editar, habilitar ou desabilitar aliases.
- Use `. $PROFILE` apos alterar a integracao instalada ou ao abrir uma nova sessao.
- Para comandos cujo executavel tenha espacos no caminho, use o operador `&`: `& 'C:\Caminho Com Espacos\tool.exe'`.
- Para arquivos `.ps1`, prefira invocacao explicita: `& 'C:\caminho\script.ps1'`.
- As definicoes continuam usando o mesmo formato `.alias`:

```text
alias gst='git status -sb'
alias pfle='& ''C:\Program Files\Notepad++\notepad++.exe'' ''C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1'''
```

### Solucao de problemas (PowerShell)

- `Get-Alias meualias` nao encontra o alias:
  Use `Get-Command meualias`. Este projeto registra aliases gerenciados como funcoes.
- `meualias` nao foi reconhecido:
  Rode `. $PROFILE` ou `alias-reload` na sessao atual do PowerShell.
- Um executavel com espacos no caminho falha ao abrir:
  Defina o alias com `&` e coloque o caminho do executavel entre aspas.
- Um alias para script `.ps1` falha ao iniciar:
  Defina explicitamente como `& 'C:\caminho\script.ps1'`. Se o ambiente bloquear execucao de scripts, ajuste a execution policy do PowerShell conforme os requisitos do seu sistema.
- Voce alterou a integracao instalada e ainda ve comportamento antigo:
  Rode `.\setup.ps1` novamente e depois `. $PROFILE`.

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
