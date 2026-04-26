defmodule CDATAFrontendWeb.Router do
  use CDATAFrontendWeb, :router

  import Phoenix.LiveDashboard.Router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {CDATAFrontendWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", CDATAFrontendWeb do
    pipe_through :browser

    live "/", DashboardLive, :index
    live "/detail/:entity_type/:entity_id", DetailLive, :show
    live "/hsc-lineage", DetailLive, :hsc_lineage
    live "/sobol", DetailLive, :sobol
  end

  if Mix.env() == :dev do
    scope "/dev" do
      pipe_through :browser

      live_dashboard "/dashboard", metrics: CDATAFrontendWeb.Telemetry
    end
  end
end