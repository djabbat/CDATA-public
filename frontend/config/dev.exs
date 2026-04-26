import Config

config :cdata_frontend, CDATAFrontendWeb.Endpoint,
  debug_errors: true,
  code_reloader: true,
  check_origin: false,
  watchers: [
    esbuild: {Esbuild, :install_and_run, [:default, ~w(--sourcemap=inline --watch)]},
    tailwind: {Tailwind, :install_and_run, [:default, ~w(--watch)]}
  ]

config :cdata_frontend, CDATAFrontendWeb.Endpoint,
  live_reload: [
    patterns: [
      ~r"priv/static/.*(js|css|png|jpeg|jpg|gif|svg)$",
      ~r"lib/cdata_frontend_web/(controllers|live|components)/.*(ex|heex)$"
    ]
  ]

config :cdata_frontend, CDATAFrontend.Repo,
  show_sensitive_data_on_connection_error: true,
  pool_size: 10

config :logger, :console, format: "[$level] $message\n"

config :phoenix_live_reload,
  dirs: [
    "priv/static",
    "lib/cdata_frontend_web"
  ]