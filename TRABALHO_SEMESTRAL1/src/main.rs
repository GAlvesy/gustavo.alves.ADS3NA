use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, FromRow};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use axum::http::StatusCode;
use html_escape::encode_text;

#[derive(Debug, Deserialize)]
struct NovoLivro {
    titulo: String,
    descricao: Option<String>,
    ano_publicacao: Option<i32>,
    fk_autor: i32,
    fk_editora: i32,
}

#[derive(FromRow)]
struct LivroComAutor {
    id: i32,
    titulo: String,
    descricao: Option<String>,
    ano_publicacao: Option<i32>,
    fk_autor: Option<i32>,
    fk_editora: Option<i32>,
    nome_autor: Option<String>,
}

#[derive(FromRow, Serialize)]
struct Autor {
    id: i32,
    nome: String,
}

#[derive(FromRow, Serialize)]
struct Editora {
    id: i32,
    nome: String,
}

#[derive(Debug, Deserialize)]
struct EscolhaLivro {
    livro_id: i32,
    usuario: String,
}

#[derive(FromRow)]
struct LivroRanking {
    titulo: String,
    total_escolhas: i64,
}

#[tokio::main]
async fn main() {
    let db = MySqlPool::connect("mysql://root:Luanzingameplay2@localhost:3306/livraria")
        .await
        .expect("Falha ao conectar no banco");

    let app = Router::new()
        .route("/", get(index).post(adicionar))
        .route("/autores", get(listar_autores))
        .route("/editoras", get(listar_editoras))
        .route("/escolher", get(form_escolha).post(receber_escolha))
        .route("/ranking", get(ranking))
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor rodando em http://{}", addr);

    let listener = TcpListener::bind(addr).await.expect("Falha ao bindar");

    axum::serve(listener, app).await.unwrap();
}

async fn index(State(db): State<MySqlPool>) -> Html<String> {
    let livros = sqlx::query_as::<_, LivroComAutor>(
        r#"
        SELECT l.id, l.titulo, l.descricao, l.ano_publicacao, l.fk_autor, l.fk_editora, a.nome AS nome_autor
        FROM livro l
        LEFT JOIN autor a ON l.fk_autor = a.id
        "#
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    let html = format!(r#"
    <!DOCTYPE html>
    <html lang="pt-BR">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Livros</title>
        <style>
            body {{
                font-family: Arial, sans-serif;
                max-width: 800px;
                margin: 40px auto;
                padding: 0 20px;
                background-color:rgb(116, 94, 52);
                color: #333;
            }}
            h1 {{
                text-align: center;
                color: #2c3e50;
            }}
            form {{
                background: #fff;
                padding: 20px;
                border-radius: 8px;
                box-shadow: 0 2px 5px rgba(0,0,0,0.1);
                margin-bottom: 40px;
            }}
            input[type="text"], textarea, input[type="number"], select {{
                width: 100%;
                padding: 10px;
                margin: 8px 0 16px 0;
                border: 1px solid #ccc;
                border-radius: 4px;
                box-sizing: border-box;
                font-size: 16px;
                resize: vertical;
            }}
            button {{
                background-color: #3498db;
                color: white;
                padding: 12px 20px;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 16px;
                transition: background-color 0.3s ease;
            }}
            button:hover {{
                background-color: #2980b9;
            }}
            table {{
                width: 100%;
                border-collapse: collapse;
                background: #fff;
                border-radius: 8px;
                overflow: hidden;
                box-shadow: 0 2px 5px rgba(0,0,0,0.1);
            }}
            th, td {{
                padding: 12px 15px;
                text-align: left;
                border-bottom: 1px solid #ddd;
            }}
            th {{
                background-color: #3498db;
                color: white;
            }}
            tr:hover {{
                background-color: #f1f1f1;
            }}
            @media (max-width: 600px) {{
                body {{
                    margin: 20px;
                    padding: 0 10px;
                }}
                table, thead, tbody, th, td, tr {{
                    display: block;
                }}
                th {{
                    position: absolute;
                    top: -9999px;
                    left: -9999px;
                }}
                tr {{
                    margin-bottom: 15px;
                }}
                td {{
                    border: none;
                    position: relative;
                    padding-left: 50%;
                }}
                td:before {{
                    position: absolute;
                    top: 12px;
                    left: 15px;
                    width: 45%;
                    padding-right: 10px;
                    white-space: nowrap;
                    font-weight: bold;
                }}
                td:nth-of-type(1):before {{ content: "Título"; }}
                td:nth-of-type(2):before {{ content: "Autor"; }}
                td:nth-of-type(3):before {{ content: "Descrição"; }}
            }}
        </style>
        <script>
            async function carregarSelects() {{
                let autores = await fetch('/autores').then(r => r.json());
                let selectAutor = document.getElementById('fk_autor');
                selectAutor.innerHTML = autores.map(a => `<option value="${{a.id}}">${{a.nome}}</option>`).join('');

                let editoras = await fetch('/editoras').then(r => r.json());
                let selectEditora = document.getElementById('fk_editora');
                selectEditora.innerHTML = editoras.map(e => `<option value="${{e.id}}">${{e.nome}}</option>`).join('');
            }}
            window.onload = carregarSelects;
        </script>
    </head>
    <body>
        <h1>Lista de Livros</h1>
        <form method="post">
            <input name="titulo" type="text" placeholder="Título do livro" required />
            <textarea name="descricao" placeholder="Descrição" rows="4"></textarea>
            <input name="ano_publicacao" type="number" placeholder="Ano de publicação" />
            <select name="fk_autor" id="fk_autor" required>
                <option value="">Selecione o autor</option>
            </select>
            <select name="fk_editora" id="fk_editora" required>
                <option value="">Selecione a editora</option>
            </select>
            <button type="submit">Adicionar Livro</button>
        </form>
        <p><a href="/escolher">Escolher um livro</a> | <a href="/ranking">Ver ranking</a></p>
        <table>
            <thead>
                <tr>
                    <th>Título</th>
                    <th>Autor</th>
                    <th>Descrição</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </body>
    </html>
    "#,
    livros.iter().map(|l| format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        encode_text(&l.titulo),
        encode_text(l.nome_autor.as_deref().unwrap_or("")),
        encode_text(l.descricao.as_deref().unwrap_or(""))
    )).collect::<String>()
    );

    Html(html)
}

async fn adicionar(State(db): State<MySqlPool>, Form(l): Form<NovoLivro>) -> Result<Redirect, (StatusCode, String)> {
    let result = sqlx::query(
        "INSERT INTO livro (titulo, descricao, ano_publicacao, fk_autor, fk_editora) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&l.titulo)
    .bind(&l.descricao)
    .bind(l.ano_publicacao)
    .bind(l.fk_autor)
    .bind(l.fk_editora)
    .execute(&db)
    .await;

    match result {
        Ok(_) => Ok(Redirect::to("/")),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao inserir: {}", e))),
    }
}

async fn listar_autores(State(db): State<MySqlPool>) -> axum::response::Json<Vec<Autor>> {
    let autores = sqlx::query_as::<_, Autor>("SELECT id, nome FROM autor")
        .fetch_all(&db)
        .await
        .unwrap_or_default();
    axum::response::Json(autores)
}

async fn listar_editoras(State(db): State<MySqlPool>) -> axum::response::Json<Vec<Editora>> {
    let editoras = sqlx::query_as::<_, Editora>("SELECT id, nome FROM editora")
        .fetch_all(&db)
        .await
        .unwrap_or_default();
    axum::response::Json(editoras)
}

async fn form_escolha(State(db): State<MySqlPool>) -> Html<String> {
 #[derive(FromRow)]  
 
struct LivroSimples {
    id: i32,
    titulo: String,
}

let livros = sqlx::query_as::<_, LivroSimples>("SELECT id, titulo FROM livro")
    .fetch_all(&db)
    .await
    .unwrap_or_default();


    let options = livros.iter()
        .map(|l| format!(r#"<option value="{}">{}</option>"#, l.id, encode_text(&l.titulo)))
        .collect::<String>();

    let html = format!(r#"
    <!DOCTYPE html>
    <html lang="pt-BR">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Escolher Livro</title>
        <style>
            body {{
                font-family: Arial, sans-serif;
                max-width: 600px;
                margin: 40px auto;
                padding: 0 20px;
                background-color: #f9f9f9;
                color: #333;
            }}
            h1 {{
                text-align: center;
                color: #2c3e50;
            }}
            form {{
                background: #fff;
                padding: 20px;
                border-radius: 8px;
                box-shadow: 0 2px 5px rgba(0,0,0,0.1);
            }}
            input[type="text"], select {{
                width: 100%;
                padding: 10px;
                margin: 8px 0 16px 0;
                border: 1px solid #ccc;
                border-radius: 4px;
                box-sizing: border-box;
                font-size: 16px;
            }}
            button {{
                background-color: #3498db;
                color: white;
                padding: 12px 20px;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 16px;
                transition: background-color 0.3s ease;
            }}
            button:hover {{
                background-color: #2980b9;
            }}
            p {{
                text-align: center;
            }}
        </style>
    </head>
    <body>
        <h1>Escolha um Livro</h1>
        <form method="post">
            <label>Seu nome:<br><input type="text" name="usuario" required></label><br><br>
            <label>Livro:<br>
                <select name="livro_id" required>
                    {}
                </select>
            </label><br><br>
            <button type="submit">Enviar Escolha</button>
        </form>
        <p><a href="/">Voltar</a></p>
    </body>
    </html>
    "#, options);

    Html(html)
}

async fn receber_escolha(State(db): State<MySqlPool>, Form(escolha): Form<EscolhaLivro>) -> Redirect {
    // Verifica se usuário já existe
  let usuario_id = if let Some(id) = sqlx::query_scalar::<_, i32>(
    "SELECT id FROM usuario WHERE nome = ? LIMIT 1"
)
.bind(&escolha.usuario)
.fetch_optional(&db)
.await
.unwrap()
{
    id
} else {
    sqlx::query_scalar::<_, i32>(
        "INSERT INTO usuario (nome, email) VALUES (?, ?)"
    )
    .bind(&escolha.usuario)
    .bind(format!("{}@example.com", escolha.usuario.replace(" ", "_")))
    .fetch_one(&db)
    .await
    .unwrap()
};


    let _ = sqlx::query(
        "INSERT INTO escolhas (livro_id, usuario_id) VALUES (?, ?)"
    )
    .bind(escolha.livro_id)
    .bind(usuario_id)
    .execute(&db)
    .await;

    Redirect::to("/ranking")
}

async fn ranking(State(db): State<MySqlPool>) -> Html<String> {
    let ranking = sqlx::query_as::<_, LivroRanking>(
        r#"
        SELECT l.titulo, COUNT(e.livro_id) AS total_escolhas
        FROM livro l
        LEFT JOIN escolhas e ON l.id = e.livro_id
        GROUP BY l.id, l.titulo
        ORDER BY total_escolhas DESC
        "#
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    let rows = ranking.iter().map(|r| {
        format!("<tr><td>{}</td><td>{}</td></tr>", encode_text(&r.titulo), r.total_escolhas)
    }).collect::<String>();

    let html = format!(r#"
        <!DOCTYPE html>
        <html lang="pt-BR">
        <head>
            <meta charset="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <title>Ranking de Livros</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    max-width: 600px;
                    margin: 40px auto;
                    padding: 0 20px;
                    background-color: #f9f9f9;
                    color: #333;
                }}
                h1 {{
                    text-align: center;
                    color: #2c3e50;
                }}
                table {{
                    width: 100%;
                    border-collapse: collapse;
                    background: #fff;
                    border-radius: 8px;
                    overflow: hidden;
                    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
                }}
                th, td {{
                    padding: 12px 15px;
                    text-align: left;
                    border-bottom: 1px solid #ddd;
                }}
                th {{
                    background-color: #3498db;
                    color: white;
                }}
                tr:hover {{
                    background-color: #f1f1f1;
                }}
                p {{
                    text-align: center;
                }}
            </style>
        </head>
        <body>
            <h1>Ranking de Livros</h1>
            <table>
                <thead>
                    <tr><th>Título</th><th>Quantidade de Escolhas</th></tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
            <p><a href="/">Voltar</a></p>
        </body>
        </html>
    "#, rows);

    Html(html)
}
