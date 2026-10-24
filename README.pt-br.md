# Gerenciador de Aliases

Uma ferramenta agnóstica de shell para gerenciar seus aliases, seguindo os padrões XDG. Um único programa (`aliases`, escrito em Rust) atende Bash, Zsh e PowerShell, no Linux, macOS, Windows e WSL.

## Funcionalidades

- **Armazenamento Padronizado**: Armazena aliases em `~/.config/aliases/aliases.d/`.
- **Modular**: Organiza aliases em arquivos separados (ex: `10-git.alias`, `20-docker.alias`).
- **Uma implementação para todos os shells**: o mesmo programa lê os arquivos e gera o código para Bash, Zsh e PowerShell.
- **Setup Idempotente**: É seguro rodar o setup várias vezes.
- **Diagnóstico**: Comando `alias-doctor` embutido.
- **Buscável**: Encontre aliases facilmente com `alias-find`.
- **Autocompletar**: `aliases completions <shell>` gera o script de completion.

## Instalação

Para compilar é preciso ter o [Rust](https://rustup.rs) (`cargo`).

### Bash / Zsh (Linux, macOS, WSL)

```bash
git clone https://github.com/seuusuario/aliases.git
cd aliases
./setup.sh
```

Isso compila o programa, instala em `~/.local/bin/aliases` (defina `ALIASES_BIN_DIR` para mudar) e adiciona este bloco ao `~/.zshrc` ou `~/.bashrc`:

```bash
# >>> aliases-manager >>>
eval "$('/home/voce/.local/bin/aliases' init zsh)"
# <<< aliases-manager <<<
```

Reinicie seu terminal ou execute `source ~/.zshrc` (ou `~/.bashrc`) para aplicar.

### PowerShell (Windows)

```powershell
git clone https://github.com/seuusuario/aliases.git
cd aliases
.\setup.ps1
. $PROFILE
```

Isso compila o `aliases.exe`, instala em `%LOCALAPPDATA%\Programs\aliases` (adicionado ao `PATH` do usuário) e adiciona o bloco de integração ao `$PROFILE`. Funciona no PowerShell 7 e no Windows PowerShell 5.1.
Os aliases gerenciados são carregados como `Function` do PowerShell. Para inspecionar um alias, use `Get-Command <nome>`. `Get-Alias` mostra apenas aliases nativos do PowerShell.

Se você rodar o setup novamente, o bloco de integração é substituído, sem duplicação.

### Windows e WSL na mesma máquina

Instale cada lado separadamente: `.\setup.ps1` no PowerShell e `./setup.sh` dentro do WSL. São dois builds do mesmo código (`aliases.exe` e um `aliases` Linux), e cada um tem seus próprios aliases: `%USERPROFILE%\.config\aliases` no Windows e `~/.config/aliases` no WSL. Em geral é isso que você quer, porque comandos como `& 'C:\Program Files\...'` só fazem sentido no PowerShell.

Para compartilhar um único conjunto de aliases, aponte `ALIASES_DIR` para a pasta do outro lado (ex.: `export ALIASES_DIR=/mnt/c/Users/voce/.config/aliases/aliases.d` no WSL). Arquivos com quebra de linha do Windows são lidos corretamente, mas ler de `/mnt/c` é mais lento.

O `alias-doctor` avisa quando o WSL só enxerga o `aliases.exe` do Windows no `PATH` (por padrão o WSL acrescenta o `PATH` do Windows).

## Uso

### Comandos

| Comando | Descrição |
|---------|-------------|
| `alias-create [nome] [cmd] [arquivo]` | Cria um novo alias. Sem argumentos, pergunta cada campo. Padrão: `local.alias`. |
| `alias-remove <nome> [arquivo]` | Remove um alias. |
| `alias-list` | Lista todos os aliases gerenciados. |
| `alias-find <termo>` | Busca aliases pelo nome ou pelo comando. |
| `alias-edit <nome>` | Edita o arquivo contendo o alias (usa `$EDITOR`). |
| `alias-reload` | Recarrega aliases imediatamente. |
| `alias-enable <nome>` | Habilita um arquivo de alias desativado. |
| `alias-disable <nome>` | Desativa um arquivo de alias. |
| `alias-doctor` | Verifica a integridade da instalação. |

Os comandos `alias-*` são funções do shell definidas pela integração. Os que alteram aliases já os recarregam no shell atual. Cada um chama um subcomando do programa `aliases`, que também pode ser usado diretamente:

```bash
aliases create gs 'git status' git   # também: aliases add
aliases remove gs                    # também: aliases rm
aliases list                         # também: aliases ls
aliases find docker
aliases edit git --path              # mostra o arquivo em vez de abrir
aliases doctor
aliases --help
```

### Criando de forma interativa

Rode `alias-create` sem argumentos e ele pergunta o nome, o comando e o arquivo, validando cada um enquanto você digita. O campo de arquivo sugere os arquivos existentes (Tab completa; um nome novo cria o arquivo), e nada é gravado antes de você confirmar. Esc cancela. Passe parte dos valores para ele perguntar só o resto: `alias-create gs`, ou `alias-create -f git` para escolher o arquivo de antemão.

```text
Create an alias  Enter confirms · Esc cancels
◇ Alias name glog
◇ Command git log --oneline
◇ File git
◇ Save to git.alias? Yes
✅ Alias 'glog' added to ~/.config/aliases/aliases.d/git.alias
```

Com todos os argumentos, ou fora de um terminal (scripts), ele nunca pergunta.

### Exemplos

```bash
# Criar um novo alias na categoria 'git'
alias-create gs 'git status' git

# Ou deixar que ele pergunte cada campo (Tab completa o nome do arquivo)
alias-create

# Listar todos os aliases
alias-list

# Buscar aliases relacionados ao docker
alias-find docker

# Ver ajuda de um comando
alias-edit --help
```

```powershell
# Criar um alias no PowerShell (ele já fica disponível)
alias-create gst "git status -sb" git

# Usar o alias normalmente
gst

# Inspecionar um alias gerenciado no PowerShell
Get-Command gst

# Usar sintaxe explícita do PowerShell para executáveis com espaço no caminho
alias-create pfle "& 'C:\Program Files\Notepad++\notepad++.exe' '$PROFILE'" system
pfle

# Executar um script PowerShell
alias-create syncdocs "& 'C:\tools\sync-docs.ps1'" scripts
syncdocs

# Executar um script PowerShell com argumentos fixos
alias-create deploy-api "& 'D:\work\deploy-api.ps1' -Environment dev -Verbose" scripts
deploy-api
```

### Autocompletar

```bash
aliases completions zsh > ~/.zfunc/_aliases     # com fpath+=~/.zfunc antes do compinit
aliases completions bash > ~/.local/share/bash-completion/completions/aliases
```

```powershell
aliases completions powershell | Out-String | Invoke-Expression
```

### Notas para PowerShell

- Os aliases gerenciados são funções do PowerShell, não entradas `AliasInfo`.
- Use `Get-Command <nome>` para verificar se um alias foi carregado.
- Use `-Help` (ou `--help`) nos comandos, como `alias-create -Help`.
- Nomes que o PowerShell não aceita como nome de função (ex.: `..`) são ignorados lá; o `alias-doctor` lista quais são.
- Se você alterar arquivos de alias fora dos comandos `alias-*`, rode `alias-reload`.
- Para comandos cujo executável tenha espaços no caminho, use o operador `&`: `& 'C:\Caminho Com Espacos\tool.exe'`.
- Para arquivos `.ps1`, prefira invocação explícita: `& 'C:\caminho\script.ps1'`.
- As definições usam o mesmo formato `.alias`:

```text
alias gst='git status -sb'
alias pfle='& ''C:\Program Files\Notepad++\notepad++.exe'' ''C:\Users\you\Documents\PowerShell\Microsoft.PowerShell_profile.ps1'''
```

### Solução de problemas (PowerShell)

- `Get-Alias meualias` não encontra o alias:
  Use `Get-Command meualias`. Este projeto registra aliases gerenciados como funções.
- `meualias` não foi reconhecido:
  Rode `. $PROFILE` ou `alias-reload` na sessão atual do PowerShell.
- Um executável com espaços no caminho falha ao abrir:
  Defina o alias com `&` e coloque o caminho do executável entre aspas.
- Um alias para script `.ps1` falha ao iniciar:
  Defina explicitamente como `& 'C:\caminho\script.ps1'`. Se o ambiente bloquear execução de scripts, ajuste a execution policy do PowerShell conforme os requisitos do seu sistema.
- Você alterou a integração instalada e ainda vê comportamento antigo:
  Rode `.\setup.ps1` novamente e depois `. $PROFILE`.

## Arquivos de alias

```
~/.config/aliases/
  aliases.d/
    00-core.alias    # Aliases principais
    local.alias      # Aliases locais/customizados
    ...
```

Arquivos são carregados em ordem alfabética. `local.alias` é sempre carregado por último, então suas definições prevalecem. Cada linha é um comentário, uma linha em branco ou uma definição em sintaxe de shell POSIX:

```text
# git
alias gs='git status'
alias say="echo \"oi\""
```

Outras linhas (funções, `export`, `alias -g`) não são carregadas; o shell mostra um aviso ao iniciar e o `alias-doctor` lista quais são.

O local segue `$ALIASES_DIR`, depois `$ALIASES_ROOT/aliases.d`, depois `$XDG_CONFIG_HOME/aliases/aliases.d`, depois `~/.config/aliases/aliases.d`.

## Desenvolvimento

- **Testes**: `cargo test` (programa), `bats tests/` (integração com Bash, depois de `cargo build`), `pwsh -File tests/smoke.ps1 -Binary target/debug/aliases.exe` (integração com PowerShell)
- **Linting**: `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `shellcheck setup.sh uninstall.sh share/init.sh`
- **Arquitetura**: veja [docs/pt-br/ARCHITECTURE.md](docs/pt-br/ARCHITECTURE.md) e [docs/DECISIONS.md](docs/DECISIONS.md)

## Desinstalação

### Bash / Zsh

Execute `./uninstall.sh` para remover a integração com o shell e o programa. Você será perguntado se deseja remover o diretório de configuração (`--remove-config` pula a pergunta).

### PowerShell

```powershell
.\uninstall.ps1
```

Isso remove o bloco `aliases-manager` do `$PROFILE`, o `aliases.exe` e sua entrada no `PATH`, e pergunta se o diretório de configuração também deve ser removido.

Para remover o diretório de configuração sem prompt extra:

```powershell
.\uninstall.ps1 -RemoveConfig
```
