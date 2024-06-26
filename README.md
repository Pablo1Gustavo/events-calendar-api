
# Agenda Colaborativa

> Implementação da proposta de trabalho para a disciplina de Banco de Dados.

## Pré-requisitos para Execução do Projeto

Antes de iniciar, certifique-se de cumprir os seguintes requisitos:

- **Rust**: Instale o Rust na versão 1.78, que é a utilizada neste projeto.
- **PostgreSQL**: Certifique-se de ter disponível uma instância do PostgreSQL, a versão 15 foi a testada e implementada neste projeto.
- **Configuração de Ambiente**: Duplique o arquivo `.env.example`, renomeando-o para `.env`, e ajuste os parâmetros conforme necessário.

## Configuração e Execução

Para preparar e executar o projeto, siga os passos abaixo:

1. **Migrations**: Execute os scripts SQL das migrations disponíveis na pasta `migrations`.
2. **Dependências**: Execute o projeto utilizando o comando `cargo run`. As dependências serão instaladas automaticamente.

### Gerenciamento de Migrations

Para um gerenciamento eficiente das migrations, instale o pacote `sqlx-cli` do Rust com o comando:
```
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

Acesse mais informações sobre o `sqlx-cli` [aqui](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md).

E na raiz do projeto, execute:
```
sqlx migrate run
```

## Uso

Confira as rotas disponíveis e exemplos de uso no arquivo Postman localizado na pasta `docs`. Certifique-se de que a variável `url` esteja corretamente configurada para o endereço e porta desejados (por padrão, `http://127.0.0.1:3000`).
