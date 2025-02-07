### Setup

Clone o repositório no diretório `home`

```bash
git clone git@github.com:juniormartinxo/aliases.git
```

Insira o código abaixo no arquivo `.zshrc` ou `.bashrc`

```bash
# Carrega todos os arquivos de aliases com extensão .alias da pasta ~/.aliases
if [ -d ~/.aliases ]; then
    for alias_file in ~/aliases/*.alias; do
        if [ -f "$alias_file" ]; then
            source "$alias_file"
        fi
    done
fi
```

### Torne o Script Executável
Dê as permissões para os arquivos de comando:

```bash
chmod +x ~/aliases/alias-create && chmod +x ~/aliases/alias-remove
```

### 2. Adicione a pasta àliases` ao `$PATH`
Para poder executar o comando alias-create de qualquer lugar, adicione o diretório ~/.aliases ao seu $PATH. Edite o arquivo .zshrc e adicione a seguinte linha:

```bash
export PATH="$HOME/aliases:$PATH"
```

Depois, recarregue o `.zshrc` ou `.bashrc`:

```bash
source ~/.zshrc
```

ou

```bash
source ~/.bashrc
```

Agora, o comando alias-create estará disponível globalmente.

### 3. Exemplo de Uso
Suponha que você deseja criar um alias chamado test com o comando cd ~/teste && clear no arquivo teste.alias. Execute o seguinte comando:

```bash
alias-create test 'cd ~/teste && clear' teste
```

### Resultado:
Se o arquivo teste.alias não existir, ele será criado.
O alias será adicionado ao arquivo:

```bash
alias test='cd ~/teste && clear'
```

O arquivo será recarregado automaticamente, e o alias estará disponível imediatamente.

Se o alias test já existir no arquivo, o script avisará:

```text
O alias 'test' já existe no arquivo 'teste.alias'.
```


### Dicas Adicionais

Localização do Script :
Se você deseja que o script esteja disponível globalmente, mova-o para `/usr/local/bin`:

```bash
sudo mv ~/.aliases/alias-create /usr/local/bin/
```

Certifique-se de que `/usr/local/bin` esteja no seu `$PATH`.

Backup Automático :
Antes de modificar um arquivo, você pode criar um backup automático:

```bash
cp "$file_path" "${file_path}.bak"
```

Comentários nos Arquivos:
Considere adicionar comentários automaticamente ao criar novos aliases. Por exemplo:

```bash
echo "# Alias adicionado em $(date)" >> "$file_path"
```

Extensão Padrão:
Mantenha a convenção de usar sempre a extensão `.alias` para arquivos de *aliases*.
