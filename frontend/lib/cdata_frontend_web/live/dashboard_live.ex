defmodule CDATAFrontendWeb.DashboardLive do
  use CDATAFrontendWeb, :live_view
  alias CDATAFrontendWeb.Clients.BackendClient
  alias CDATAFrontendWeb.CoreComponents

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket) do
      send(self(), :load_data)
    end

    socket =
      socket
      |> assign(
        page_title: "CDATA Dashboard",
        loading: true,
        concept: nil,
        parameters: [],
        axioms: [],
        error: nil
      )
      |> assign_metadata()

    {:ok, socket}
  end

  @impl true
  def handle_info(:load_data, socket) do
    socket =
      case BackendClient.fetch_concept() do
        {:ok, concept} ->
          axioms = extract_axioms(concept)
          assign(socket, concept: concept, axioms: axioms, loading: false)

        {:error, error} ->
          assign(socket, error: "Failed to load concept: #{inspect(error)}", loading: false)
      end

    socket =
      case BackendClient.fetch_parameters() do
        {:ok, parameters} ->
          assign(socket, parameters: parameters)

        {:error, error} ->
          put_flash(socket, :error, "Failed to load parameters: #{inspect(error)}")
      end

    {:noreply, socket}
  end

  @impl true
  def handle_event("view_detail", %{"entity_type" => type, "entity_id" => id}, socket) do
    {:noreply, push_navigate(socket, to: ~p"/detail/#{type}/#{id}")}
  end

  def render(assigns) do
    ~H"""
    <div class="space-y-8">
      <CoreComponents.loading_container loading?={@loading}>
        <%= if @error do %>
          <div class="rounded-md bg-red-50 p-4 mb-6">
            <div class="flex">
              <div class="flex-shrink-0">
                <Heroicons.exclamation_triangle class="h-5 w-5 text-red-400" />
              </div>
              <div class="ml-3">
                <h3 class="text-sm font-medium text-red-800">Data loading error</h3>
                <div class="mt-2 text-sm text-red-700">
                  <p><%= @error %></p>
                </div>
              </div>
            </div>
          </div>
        <% end %>

        <.impact_statement />

        <.executive_summary summary={@concept && @concept["executive_summary"]} />

        <.axioms_section axioms={@axioms} />

        <.parameters_section parameters={@parameters} />

        <.counter_registry />

        <.hallmark_quote />
      </CoreComponents.loading_container>
    </div>
    """
  end

  defp impact_statement(assigns) do
    ~H"""
    <CoreComponents.section title="Impact Statement">
      <div class="prose max-w-none">
        <p class="text-lg text-gray-700">
          All existing clinical tools of biological age (GrimAge, DunedinPACE, PhenoAge) are diagnostic, not therapeutic. They answer "how fast are you aging?" but not "what exactly to stop?". CDATA is the only theory answering the second question through logical necessity: PTM degradation of centrioles. This transforms CDATA from an academic theory into a roadmap for developing the first class of anti-aging drugs with a specific molecular target—deglutamylases—which would not be identified by any empirical screening, as they are not part of any of the 12 Hallmarks of Aging and are not predicted by any existing aging theory.
        </p>
      </div>
    </CoreComponents.section>
    """
  end

  defp executive_summary(assigns) do
    ~H"""
    <CoreComponents.section title="Executive Summary">
      <div class="prose max-w-none">
        <%= if @summary do %>
          <p><%= @summary %></p>
        <% else %>
          <p>
            CDATA (Centriolar Damage Accumulation Theory of Aging) is a mechanistic aging theory explaining organismal degradation as an inevitable consequence of PTM damage accumulation in stem cell mother centrioles.
          </p>
          <ul>
            <li><strong>32 parameters</strong> (reduced from 120)</li>
            <li><strong>8 key mechanisms</strong> (with real PMIDs for 23/32 parameters)</li>
            <li><strong>R²(MCAI)=0.745, R²(CHIP)=0.611, R²(Telo)=0.465</strong> (in-sample cross-sectional fit)</li>
            <li><strong>TRL 3→4</strong> positioning</li>
            <li><strong>10 falsifiable predictions</strong> (P1–P6 + P7–P10)</li>
          </ul>
        <% end %>
      </div>
    </CoreComponents.section>
    """
  end

  defp axioms_section(assigns) do
    ~H"""
    <CoreComponents.section title="CDATA Axioms — DO NOT MODIFY WITHOUT SPECIAL COMMAND">
      <div class="space-y-6">
        <p class="text-sm text-red-600 italic">
          These three statements are the foundation of CDATA theory. They must not be changed, revised, or removed without explicit user command. They must be present in all LOIs, grants, papers, and public CDATA materials.
        </p>
        <div :for={axiom <- @axioms} class="space-y-4">
          <CoreComponents.axiom_card
            id={axiom["id"]}
            title={axiom["title"]}
            number={axiom["number"]}
          >
            <p><%= axiom["mechanism"] %></p>
            <p class="mt-2 text-xs">
              <strong>Key references:</strong> <%= axiom["references"] %>
            </p>
          </CoreComponents.axiom_card>
        </div>
      </div>
    </CoreComponents.section>
    """
  end

  defp parameters_section(assigns) do
    ~H"""
    <CoreComponents.section title="Quantitative Parameters (Reduced Model)">
      <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
        <CoreComponents.parameter_card
          :for={param <- @parameters}
          id={param["symbol"]}
          label={param["label"]}
          value={param["value"]}
          unit={param["unit"]}
          description={param["description"]}
          status={param["status"]}
        />
      </div>
      <div class="mt-6">
        <.link
          navigate={~p"/detail/parameters/all"}
          class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700"
        >
          View all parameters
        </.link>
      </div>
    </CoreComponents.section>
    """
  end

  defp counter_registry(assigns) do
    ~H"""
    <CoreComponents.section title="MCOA Counter Registry">
      <div class="space-y-4">
        <div class="bg-blue-50 border border-blue-200 rounded-md p-4">
          <div class="flex">
            <div class="flex-shrink-0">
              <Heroicons.information_circle class="h-5 w-5 text-blue-400" />
            </div>
            <div class="ml-3">
              <h3 class="text-sm font-medium text-blue-800">MCOA Framework</h3>
              <div class="mt-2 text-sm text-blue-700">
                <p>CDATA is Counter #2 (Centriolar) in the Multi-Counter Architecture of Organismal Aging. Connection coefficient γ_CDATA = 0 (null hypothesis per CORRECTIONS §1.3).</p>
              </div>
            </div>
          </div>
        </div>

        <CoreComponents.table headers={["Counter", "Weight w_i(tissue)", "Kinetics", "Status"]}>
          <tr :for={counter <- mcoa_counters()}>
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
              <%= counter.name %>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              <%= counter.weight %>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              D(t) = D₀ + α·(n/n*) + β·(t/τ)
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <CoreComponents.badge type={counter.status_type}>
                <%= counter.status %>
              </CoreComponents.badge>
            </td>
          </tr>
        </CoreComponents.table>
      </div>
    </CoreComponents.section>
    """
  end

  defp hallmark_quote(assigns) do
    ~H"""
    <div class="bg-gradient-to-r from-purple-50 to-blue-50 border-l-4 border-purple-500 p-4 rounded-r">
      <blockquote class="text-lg italic text-gray-700">
        "Centrosome misorientation is an officially recognized hallmark of stem cell aging"
        <footer class="mt-2 text-sm not-italic text-gray-600">
          — Rando, Brunet & Goodell, <em>Cell Stem Cell</em> 2025
        </footer>
      </blockquote>
      <p class="mt-3 text-gray-600">
        CDATA provides the first quantitative molecular mechanism for this hallmark: PTM accumulation in the mother centriole is the upstream driver of centrosome misorientation.
      </p>
    </div>
    """
  end

  defp assign_metadata(socket) do
    assign(socket,
      version: "5.2",
      canonical_date: "2026-04-22",
      backend_url: System.get_env("BACKEND_URL", "http://localhost:3003")
    )
  end

  defp extract_axioms(concept) do
    case concept do
      %{"axioms" => axioms} when is_list(axioms) -> axioms
      _ -> default_axioms()
    end
  end

  defp default_axioms do
    [
      %{
        "id" => "axiom_1",
        "number" => 1,
        "title" => "Hayflick in hypoxia with telomerase",
        "mechanism" => "Stem cells in hypoxic environment with active telomerase still reach Hayflick limit.",
        "references" => "Harrison & Astle, JEM 1982; Allsopp et al., JEM 2003"
      },
      %{
        "id" => "axiom_2",
        "number" => 2,
        "title" => "Defective cilia signaling from old mother centriole",
        "mechanism" => "Mother centriole inherited by stem daughter is basal body of primary cilium. PTM accumulation degrades ciliary signaling.",
        "references" => "Whitfield et al., Cell Reports 2016; Gao et al., Nature 2009"
      },
      %{
        "id" => "axiom_3",
        "number" => 3,
        "title" => "Reduced division rate with old centriole",
        "mechanism" => "Division rate of stem cells with inherited old PTM-loaded centrioles decreases over time.",
        "references" => "Wilson et al., Nature 2008; Kowalczyk et al., Cell Stem Cell 2015"
      }
    ]
  end

  defp mcoa_counters do
    [
      %{name: "Counter #1 (Telomeric)", weight: "w₁(tissue)", status: "Validated", status_type: "success"},
      %{name: "Counter #2 (Centriolar) - CDATA", weight: "w₂(tissue)", status: "Phase 0", status_type: "warning"},
      %{name: "Counter #3 (Epigenetic)", weight: "w₃(tissue)", status: "Planned", status_type: "info"},
      %{name: "Counter #4 (Proteostatic)", weight: "w₄(tissue)", status: "Planned", status_type: "info"}
    ]
  end
end