# Arquitetura

A ferramenta é um programa Rust, `aliases`, mais scripts de instalação finos. Um programa não consegue alterar o shell que o executa, então o shell avalia o código que o programa imprime (`eval "$(aliases init zsh)"`), a mesma abordagem do `zoxide` e do `starship` (ADR-005).

## O programa (`src/`)

| Módulo | Responsabilidade |
|--------|------------------|
| `main.rs` | Linha de comando (`clap`): subcomandos, textos de ajuda, códigos de saída. |
| `alias_file.rs` | O formato `.alias`: interpreta cada linha (valores com aspas simples, duplas ou sem aspas, escapes `'\''`, CRLF, BOM UTF-8/UTF-16), a ordem de carga e as edições (acrescentar, remover com backup `.bak`, escrita atômica que preserva permissões). |
| `init.rs` | `aliases init <shell>`: o código que cada shell avalia. Bash/Zsh recebem linhas `alias`; o PowerShell recebe uma função por alias, escrita só em ASCII para qualquer codificação de console ler. Ambos recebem também os wrappers `alias-*` e guardam quais aliases carregaram, para o `alias-reload` remover os que sumiram. |
| `commands.rs` | `create`, `remove`, `list`, `find`, `edit`, `enable`, `disable`. |
| `list.rs` | O layout do `list`: um bloco por arquivo, até 3 colunas conforme a largura do terminal, comandos truncados com `…`. |
| `install.rs` | `setup` / `uninstall`: o bloco marcado no `.bashrc`, `.zshrc` ou `$PROFILE` (ADR-002), detecção do shell, `00-core.alias` padrão. |
| `doctor.rs` | `doctor`: diretório de configuração, linhas não suportadas, aliases definidos em mais de um arquivo, nomes que o PowerShell não carrega, integração com o shell, programa no `PATH` (incluindo o caso do WSL em que só o `aliases.exe` aparece). |
| `paths.rs` | Caminhos da configuração (ADR-001). |
| `ui.rs` | Mensagens de status, cores (só em terminal, respeitando `NO_COLOR`) e o tipo de erro. |

## Integração com o shell

`aliases init <shell>` imprime:

1. O caminho absoluto do programa, para os wrappers funcionarem mesmo fora do `PATH`.
2. As funções `alias-*`. As que alteram arquivos (`alias-create`, `alias-remove`, `alias-edit`, `alias-enable`, `alias-disable`) chamam o programa e depois o `alias-reload`. Elas definem `ALIASES_MANAGER_WRAPPED=1` para o programa não imprimir a dica "rode alias-reload".
3. Os aliases: primeiro os da carga anterior são removidos, depois os atuais são definidos. O `alias-reload` roda `aliases init <shell> --aliases-only`, que imprime só esta parte.

## Scripts da raiz

### `setup.sh` / `setup.ps1`
- Compilam o programa com `cargo build --release`.
- Instalam (`~/.local/bin/aliases`, ou `%LOCALAPPDATA%\Programs\aliases\aliases.exe`, adicionado ao `PATH` do usuário).
- Rodam `aliases setup`, que cria o diretório de configuração e escreve o bloco de integração.

### `uninstall.sh` / `uninstall.ps1`
- Rodam `aliases uninstall`, que remove o bloco e, opcionalmente, a configuração.
- Removem o programa instalado.

## `share/`

`share/init.sh` e `share/init.ps1` são shims de compatibilidade para arquivos de inicialização que ainda têm o bloco antigo `source .../share/init.sh`: eles encontram o programa e carregam a saída do `init`. Rodar o setup de novo substitui o bloco antigo. Serão removidos numa versão futura.

## `tests/`

- `cli.rs`: o programa de ponta a ponta, num diretório de configuração temporário (`cargo test`).
- `test_basic.bats`: sessões Bash que carregam a integração (`bats tests/`).
- `smoke.ps1`: o mesmo para PowerShell 7 e Windows PowerShell 5.1.
