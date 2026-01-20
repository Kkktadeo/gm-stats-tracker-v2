use std::{str::FromStr, sync::OnceLock};

use base64::{Engine, engine::general_purpose};
use resvg::{render, tiny_skia::Pixmap, usvg::{self, Transform}};
use serde::Deserialize;
use worker::{Cache, Context, Env, Fetch, Headers, Request, Response, Result, Url, event};

#[derive(Deserialize)]
struct GMSGameStatus {
    title: String,
    connected: u16,
    id: u16
}

#[derive(Deserialize)]
struct GMSNodeStatus {
    games: Vec<GMSGameStatus>
}

#[derive(Deserialize)]
struct GMSStatusResponse {
    status: Vec<GMSNodeStatus>
}

/// Fetches (or uses cached) games status from gamemakerserver
async fn fetch_status(ctx: &Context) -> Result<GMSStatusResponse> {
    let url = Url::from_str("https://gamemakerserver.com/dynamic/status.php")?;
    
    match Cache::default().get(&url.to_string(), false).await? {
        Some(mut resp) => resp.json::<GMSStatusResponse>().await,
        None => {
            let mut resp = Fetch::Url(url.clone()).send().await?;

            let mut cache_resp = resp.cloned()?;
            cache_resp.headers_mut().set("Cache-Control", "public, max-age=300")?;

            ctx.wait_until(async move {
                let _ = Cache::default().put(&url.to_string(), cache_resp).await;
            });

            resp.json::<GMSStatusResponse>().await
        },
    }
}

/// Fetches screenshot of gamemaker game
async fn fetch_screenshot(game_id: u16) -> Result<Option<String>> {
    let html = Fetch::Url(format!("https://gamemakerserver.com/en/games/{game_id}").parse()?)
        .send().await?
        .text().await?;

    let result = match html
        .split_once("/thumb-screenshots/").unwrap_or_default().1
        .split_once("/") {
            Some((screenshot_id, _)) => {
                let bytes = Fetch::Url(format!("https://gamemakerserver.com/thumb-screenshots/{screenshot_id}/").parse()?)
                    .send().await?
                    .bytes().await?;
        
                Some(format!("data:image/jpeg;base64,{}", general_purpose::STANDARD.encode(bytes)))
            },
            None => None,
        };
    
    Ok(result)
}

static USVG_OPTIONS: OnceLock<usvg::Options> = OnceLock::new();
fn get_usvg_options() -> &'static usvg::Options<'static> {
    USVG_OPTIONS.get_or_init(|| {
        let mut opts = usvg::Options::default();
        opts.fontdb_mut().load_font_data(include_bytes!("./assets/fonts/RobotoCondensedMini.woff2").to_vec());
        opts
    })
}

/// Generates image from game using template SVG
async fn generate_image(game: &GMSGameStatus, env: Env) -> Result<Vec<u8>> {
    let cached_kv = env.kv("cached_images")?;

    let image_bytes = match cached_kv.get(&game.id.to_string()).bytes().await? {
        Some(bytes) => bytes,
        None => {
            let screenshot = fetch_screenshot(game.id).await?;

            let opt = get_usvg_options();
            let svg_data = format!(
                include_str!("./assets/template.svg"),
                count = game.connected.to_string(),
                title = game.title,
                screenshot = screenshot.unwrap_or_default()
            );

            let tree = usvg::Tree::from_str(
                &svg_data,
                &opt
            ).map_err(|e| e.to_string())?;

            let pixmap_size = tree.size().to_int_size();
            let mut pixmap = Pixmap::new(pixmap_size.width(), pixmap_size.height()).ok_or("couldn't create pixmap")?;

            render(&tree, Transform::default(), &mut pixmap.as_mut());

            let mut bytes = Vec::new();
            image_webp::WebPEncoder::new(&mut bytes)
                .encode(pixmap.data(), pixmap_size.width(), pixmap_size.height(), image_webp::ColorType::Rgba8).map_err(|e| e.to_string())?;

            cached_kv.put_bytes(&game.id.to_string(), &bytes)?
                .expiration_ttl(300)
                .execute().await?;

            bytes
        },
    };

    Ok(image_bytes)
}

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    ctx: Context,
) -> Result<Response> {
    if req.path() != "/count" {
        return Response::error("Not found", 404);
    }

    #[derive(serde::Deserialize)]
    struct Query {
        #[serde(rename = "gameid")]
        game_id: u16
    }

    let Ok(Query { game_id }) = req.query::<_>() else {
        return Response::error("Invalid arguments!", 400)
    };

    
    let status = fetch_status(&ctx).await?;
    let Some(game) = status.status
        .iter().flat_map(|node| &node.games)
        .find(|g| g.id == game_id) else {
            return Response::error("Game not found or has no players", 404)
        };
    
    let image = generate_image(&game, env).await?;

    let headers = Headers::new();
    headers.set("Content-Type", "image/webp")?;

    Ok(Response::from_bytes(image)?.with_headers(headers))
}