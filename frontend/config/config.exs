import Config

config :cdata_frontend, CDATAFrontendWeb.Endpoint,
  url: [host: "localhost"],
  render_errors: [
    formats: [html: CDATAFrontendWeb.ErrorHTML, json: CDATAFrontendWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: CDATAFrontend.PubSub,
  live_view: [signing_salt: "vzN0EABV"]

config :cdata_frontend, CDATAFrontendWeb.Clients.BackendClient,
  base_url: System.get_env("BACKEND_URL", "http://localhost:3003"),
  timeout: 30_000,
  pool_size: 10

config :cdata_frontend, :telemetry,
  metrics_prefix: "cdata.frontend",
  enable_live_dashboard: true

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

config :esbuild,
  version: "0.19.5",
  default: [
    args: ~w(js/app.js --bundle --target=es2020 --outdir=../priv/static/assets --external:/fonts/* --external:/images/*),
    cd: Path.expand("../assets", __DIR__),
    env: %{"NODE_PATH" => Path.expand("../deps", __DIR__)}
  ]

config :tailwind,
  version: "3.4.0",
  default: [
    args: ~w(
      --config=tailwind.config.js
      --input=css/app.css
      --output=../priv/static/assets/app.css
    ),
    cd: Path.expand("../assets", __DIR__)
  ]

config :cdata_frontend, CDATAFrontend.Repo,
  database: "cdata_frontend_dev",
  username: "postgres",
  password: "postgres",
  hostname: "localhost",
  pool_size: 10

import_config "#{config_env()}.exs"