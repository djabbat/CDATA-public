import Config

if config_env() == :prod do
  database_url =
    System.get_env("DATABASE_URL") ||
      raise """
      DATABASE_URL environment variable is not set.
      For example: ecto://USER:PASS@HOST/DATABASE
      """

  config :cdata_frontend, CDATAFrontend.Repo,
    url: database_url,
    pool_size: String.to_integer(System.get_env("POOL_SIZE", "10"))

  secret_key_base =
    System.get_env("SECRET_KEY_BASE") ||
      raise """
      SECRET_KEY_BASE environment variable is not set.
      You can generate one by calling: mix phx.gen.secret
      """

  host = System.get_env("PHX_HOST", "example.com")

  config :cdata_frontend, CDATAFrontendWeb.Endpoint,
    secret_key_base: secret_key_base,
    url: [host: host, port: 4003]

  config :cdata_frontend, CDATAFrontendWeb.Clients.BackendClient,
    base_url: System.fetch_env!("BACKEND_URL"),
    timeout: 45_000
end