# Desktop proxy console

Prebuilt packages are published in [GitHub Releases](https://github.com/bibiparrot/bibillmproxy/releases). Windows has a portable ZIP, setup EXE, and MSI. Linux x86_64 and aarch64 each have an AppImage, RPM, and portable tar.gz. macOS Intel and Apple Silicon each have a ZIP, DMG, and PKG. The packages include the desktop console (`lite-gui`) and command-line server (`lite`). macOS packages are unsigned and may need to be allowed in System Settings after download.

Build and run the egui console from the repository root:

```powershell
cargo run --features gui --bin lite-gui
```

Add an OpenAI Responses-compatible or Anthropic Messages-compatible upstream in **代理路由**. Set a local model name, the upstream model (`openai/model-name` or `anthropic/model-name`), the upstream API base, and the upstream API key. The base may end in `/v1`; the gateway appends the correct protocol path. Multiple exact model names can point to different upstream proxies.

The key fields accept `os.environ/VARIABLE_NAME`. Set those variables in your environment or a `.env` file before launching the console. The **完整 YAML** tab lets you edit the full LiteLLM-compatible configuration, including other gateway settings. The form preserves other YAML sections and extra route fields, but rewriting through the form removes YAML comments.

Choose a local port, save, and start the proxy. The console binds to `127.0.0.1`. It shows the local endpoint URLs when ready. Stopping waits for the Axum server to shut down gracefully. Changes to the YAML or port take effect after a restart. The console stores its config path, port, and language choice in the per-user `settings.toml` directory selected by the operating system; model routes and secrets remain in the separate LiteLLM-compatible `config.yaml` or environment. The language selector defaults to the system locale and supports Simplified Chinese, English, Japanese, Korean, Russian, French, Spanish, Portuguese, Italian, and Romanian.

OpenAI Responses clients use `http://127.0.0.1:4000/v1/responses` (or the selected port). Claude Code should use `ANTHROPIC_BASE_URL=http://127.0.0.1:4000` and `ANTHROPIC_AUTH_TOKEN=<local master key>`. If no local master key is configured, the local API does not require authentication.

The upstream proxy must implement the selected protocol. OpenAI routes forward Responses requests; Anthropic routes forward Messages requests. The gateway does not convert a Responses request into Chat Completions or a Messages request into Responses.
