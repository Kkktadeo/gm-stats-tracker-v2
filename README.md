# gm-stats-tracker-v2
> [!IMPORTANT]
> **GameMakerServer is closing in 2027**, meaning the code will most likely be kept archived as a fun fact for others that want to create similar project in future.

GM Stats Tracker (v2) is a web service for Discord that provides up-to-date info about player count for GM game that uses [GMServer](https://gamemakerserver.com).

This is a newer and more performant version of previous gms-stats-tracker made by me **2 years ago**!

## Why more performant?
Unlike previous version that was using cron job and required you to have Node.JS runtime installed, this one is written in Rust and fits completely on a **Free Cloudflare Worker**!

That means you can host it completely for free, although it is recommended to use public instance as it'll be always up to date and won't overflow GMServer with requests!


## How it works?
The project uses Rust to be able to utilize rendering engine in it's full speed (something that wouldn't EVER be possible with pure JS implementation).

It internally uses `resvg` crate for setting up template (can be found in [template.svg](/src/assets/template.svg)) and then converts svg to webp using `image-webp` crate to be able to render it in embed.

The implementation stores as much as possible inside of Cloudflare's KV Store and Cache API to ensure amount of requests is at minimum. Which is why it updates every 5 minutes **(with possible time change, although it's not recommended to not spam both you and gmserver with requests)**

It also ensures that it brings heavily optimized fonts (for example Roboto Condensed with stipped unneeded glyphs + converted to woff2) and compiles to WASM heavily optimized for size and stripped from unneeded calls. ([Cloudflare Worker's size for free tier is 3MB](https://developers.cloudflare.com/workers/platform/limits/#account-plan-limits), as of writing this final size of project is ~1.9MB)


## Public Instance
Currently the public instance that can be used by others is `https://gm-stats-tracker-v2.jakeayy.workers.dev`, for documentation on how to use it please look below.

### How to use?
| Endpoint | Params | Body Info |
|---|---|---|
| `/count` | `?gameid=id`<br>**gameid** - Game ID, can be obtained through [GMServer](https://gamemakerserver.com/en/games/) website. | `-` |


## Development
### Prerequisites
1. A recent version of [`Rust`](https://rust-lang.org)

2. [`npm`](https://nodejs.org/) with it's `npx` (although I recommend [`bun`](https://bun.sh) and it's `bunx`), if you'll use `npx`, replace `bunx` with it in next commands.

3. `wasm32-unknown-unknown` WASM toolchain to be able to compile the worker at all.
    ```sh
    rustup target add wasm32-unknown-unknown
    ```

### Commands
- **Development**
    ```sh
    bunx wrangler dev
    ```

- **Deployment**
    ```sh
    bunx wrangler deploy
    ```
    **REMEMBER: Before deploying, ensure you replaced all needed ids with your own in [wrangler.toml](/wrangler.toml)!**

## TODO
- [ ] Proper status failure handling (currently returns 500 error if GMServer returns broken body)
- [ ] Player extended history - Shows history of previous player counts (5 -> 2 -> 6 -> 9) and indicator showing if it's more or less than previously
- [ ] Random game screenshot
- [ ] Possibly more optimizations(?)

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The fonts used in this project are licensed under the SIL Open Font License (OFL). See [src/assets/fonts/OFL.txt](src/assets/fonts/OFL.txt) for more information.
