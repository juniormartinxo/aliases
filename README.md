# **Aliases Manager** 🚀
Uma ferramenta simples e eficiente para gerenciar seus aliases no Zsh ou Bash. Com esta solução, você pode criar, remover e organizar aliases de forma dinâmica e automatizada. 💻✨

---

## **Setup** 🔧

### 1. Clone o Repositório 📥
Clone este repositório para o diretório `home`:

```bash
git clone git@github.com:juniormartinxo/aliases.git
```

> **Nota**: O diretório será criado como `aliases` (oculto) no seu diretório home. 🏠

---

### 2. Configure o Carregamento Automático de Aliases ⚡
Adicione o seguinte código ao seu arquivo `.zshrc` ou `.bashrc` para carregar automaticamente todos os arquivos de aliases com a extensão `.alias`:

```bash
# Carrega todos os arquivos de aliases com extensão .alias da pasta ~/aliases
if [ -d ~/aliases ]; then
    for alias_file in ~/aliases/*.alias; do
        if [ -f "$alias_file" ]; then
            source "$alias_file"
        fi
    done
fi
```

> **Dica**: Certifique-se de que o diretório `~/aliases` existe antes de adicionar este código. 📂

---

### 3. Torne os Scripts Executáveis 🛠️
Dê permissão de execução aos scripts `alias-create` e `alias-remove`:

```bash
chmod +x ~/aliases/alias-create ~/aliases/alias-remove
```

---

### 4. Adicione o Diretório ao `$PATH` 🌐
Para usar os comandos `alias-create` e `alias-remove` globalmente, adicione o diretório `~/aliases` ao seu `$PATH`. Insira a seguinte linha no seu `.zshrc` ou `.bashrc`:

```bash
export PATH="$HOME/aliases:$PATH"
```

Depois, recarregue o arquivo de configuração:

```bash
source ~/.zshrc
```
ou
```bash
source ~/.bashrc
```

> **Alternativa Global**: Se preferir, mova os scripts para `/usr/local/bin`: 📦
>
> ```bash
> sudo mv ~/aliases/alias-create /usr/local/bin/
> sudo mv ~/aliases/alias-remove /usr/local/bin/
> ```

---

## **Exemplo de Uso** 🎯

### Criar um Novo Alias ✨
Suponha que você deseja criar um alias chamado `test` com o comando `cd ~/teste && clear` no arquivo `teste.alias`. Execute o seguinte comando:

```bash
alias-create test 'cd ~/teste && clear' teste
```

#### Resultado:
- Se o arquivo `teste.alias` não existir, ele será criado. 🆕
- O alias será adicionado ao arquivo:
  ```bash
  # Alias adicionado em Thu Oct 19 14:00:00 UTC 2023
  alias test='cd ~/teste && clear'
  ```
- O arquivo será recarregado automaticamente, e o alias estará disponível imediatamente. 🔄

Se o alias já existir no arquivo, o script avisará:
```text
O alias 'test' já existe no arquivo 'teste.alias'. ❗
```

---

### Remover um Alias 🗑️
Para remover o alias `test` do arquivo `teste.alias`, execute:

```bash
alias-remove test teste
```

#### Resultado:
- O alias e o comentário associado serão removidos do arquivo. ✂️
- O arquivo será recarregado automaticamente. 🔄

---

## **Dicas Adicionais** 💡

### 1. Backup Automático 📋
Antes de modificar um arquivo, você pode criar um backup automático adicionando o seguinte comando ao script:

```bash
cp "$file_path" "${file_path}.bak"
```

Isso cria uma cópia de segurança com a extensão `.bak`. 🛡️

---

### 2. Comentários Automáticos 📝
Os scripts adicionam automaticamente comentários com a data e hora ao criar novos aliases. Exemplo:

```bash
# Alias adicionado em Thu Oct 19 14:00:00 UTC 2023
alias test='cd ~/teste && clear'
```

Se desejar personalizar o formato da data, modifique o comando `$(date)` no script. Por exemplo:

```bash
echo "# Alias adicionado em $(date '+%Y-%m-%d %H:%M:%S')" >> "$file_path"
```

---

### 3. Extensão Padrão 📁
Mantenha a convenção de usar sempre a extensão `.alias` para arquivos de aliases. Isso garante consistência e facilita a manutenção. 🔧

---

### 4. Organização por Categoria 🗂️
Recomenda-se organizar os arquivos de aliases por categoria. Por exemplo:
- `git.alias`: Aliases relacionados ao Git. 🌟
- `docker.alias`: Aliases para Docker. 🐳
- `system.alias`: Aliases para comandos do sistema. 💻

---

## **Contribuições** 🤝
Contribuições são bem-vindas! Se você tiver sugestões ou melhorias, sinta-se à vontade para abrir uma issue ou enviar um pull request. 🚀

---

## **Licença** 📜
Este projeto está licenciado sob a [MIT License](LICENSE). 🌐
