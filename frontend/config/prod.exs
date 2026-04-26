import Config

config :cdata_frontend, CDATAFrontendWeb.Endpoint,
  url: [host: System.get_env("PHX_HOST", "example.com"), port: 4003],
  cache_static_manifest: "priv/static/cache_manifest.json",
  force_ssl: [hsts: true],
  http: [
    port: String.to_integer(System.get_env("PORT", "4003")),
    transport_options: [socket_opts: [:inet6]]
  ]

config :cdata_frontend, CDATAFrontend.Repo,
  url: System.get_env("DATABASE_URL"),
  pool_size: String.to_integer(System.get_env("POOL_SIZE", "10")),
  ssl: true

config :cdata_frontend, CDATAFrontendWeb.Clients.BackendClient,
  base_url: System.fetch_env!("BACKEND_URL"),
  timeout: 60_000,
  pool_size: 20

config :logger, level: :info

config :phoenix, :serve_endpoints, true