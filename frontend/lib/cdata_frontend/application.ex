defmodule CDATAFrontend.Application do
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      CDATAFrontend.Repo,
      CDATAFrontend.PubSub,
      {Phoenix.PubSub, name: CDATAFrontend.PubSub},
      CDATAFrontendWeb.Endpoint,
      CDATAFrontendWeb.Telemetry,
      {Oban, Application.fetch_env!(:cdata_frontend, Oban)},
      {DynamicSupervisor, strategy: :one_for_one, name: CDATAFrontend.DynamicSupervisor}
    ]

    opts = [strategy: :one_for_one, name: CDATAFrontend.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  def config_change(changed, _new, removed) do
    CDATAFrontendWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end