defmodule CDATAFrontendWeb.Telemetry do
  use Supervisor
  import Telemetry.Metrics

  def start_link(arg) do
    Supervisor.start_link(__MODULE__, arg, name: __MODULE__)
  end

  @impl true
  def init(_arg) do
    children = [
      {:telemetry_poller, measurements: periodic_measurements(), period: 10_000}
    ]

    Supervisor.init(children, strategy: :one_for_one)
  end

  def metrics do
    [
      counter("cdata.frontend.page_views.total"),
      counter("cdata.frontend.api_requests.total"),
      counter("cdata.frontend.api_errors.total"),
      summary("cdata.frontend.api_response_time.milliseconds",
        unit: {:native, :millisecond}
      ),
      summary("cdata.frontend.live_view.mount_time.milliseconds"),
      last_value("cdata.frontend.memory.total", unit: :byte)
    ]
  end

  defp periodic_measurements do
    [
      {__MODULE__, :dispatch_api_metrics, []}
    ]
  end

  def dispatch_api_metrics do
    :telemetry.execute([:cdata, :frontend, :periodic], %{status: :ok})
  end
end