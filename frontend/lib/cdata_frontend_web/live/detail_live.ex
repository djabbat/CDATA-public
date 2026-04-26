defmodule CDATAFrontendWeb.DetailLive do
  use CDATAFrontendWeb, :live_view
  alias CDATAFrontendWeb.Clients.BackendClient
  alias CDATAFrontendWeb.CoreComponents

  @impl true
  def mount(%{"entity_type" => "parameters", "entity_id" => "all"}, _session, socket) do
    send(self(), :load_all_parameters)
    {:ok, assign(socket, page_title: "All Parameters", parameters: [], loading: true)}
  end

  def mount(%{"entity_type" => "axiom", "entity_id" => id}, _session, socket) do
    send(self(), {:load_axiom, id})
    {:ok, assign(socket, page_title: "Axiom Detail", axiom: nil, loading: true)}
  end

  def mount(_params, _session, socket) do
    {:ok, assign(socket, page_title: "Detail View")}
  end

  def mount(:sobol, _session, socket) do
    send(self(), :load_sobol_data)
    {:ok, assign(socket, page_title: "Sobol Sensitivity Analysis", sobol_data: nil, loading: true)}
  end

  def mount(:hsc_lineage, _session, socket) do
    send(self(), :load_hsc_lineage)
    {:ok, assign(socket, page_title: "HSC Lineage Tracking", lineage_data: nil, loading: true)}
  end

  @impl true
  def handle_info(:load_all_parameters, socket) do
    case BackendClient.fetch_parameters() do
      {:ok, parameters} ->
        {:noreply, assign(socket, parameters: parameters, loading: false)}

      {:error, error} ->
        {:noreply,
         socket
         |> put_flash(:error, "Failed to load parameters: #{inspect(error)}")
         |> assign(loading: false)}
    end
  end

  def handle_info({:load_axiom, id}, socket) do
    case BackendClient.fetch_concept() do
      {:ok, concept} ->
        axiom = find_axiom_by_id(concept, id)
        {:noreply, assign(socket, axiom: axiom, loading: false)}

      {:error, error} ->
        {:noreply,
         socket
         |> put_flash(:error, "Failed to load axiom: #{inspect(error)}")
         |> assign(loading: false)}
    end
  end

  def handle_info(:load_sobol_data, socket) do
    case BackendClient.fetch_sobol() do
      {:ok, data} ->
        {:noreply, assign(socket, sobol_data: data, loading: false)}

      {:error, error} ->
        {:noreply,
         socket
         |> put_flash(:error, "Failed to load Sobol data: #{inspect(error)}")
         |> assign(loading: false, sobol_data: nil)}
    end
  end

  def handle_info(:load_hsc_lineage, socket) do
    case BackendClient.fetch_hsc_lineage() do
      {:ok, data} ->
        {:noreply, assign(socket, lineage_data: data, loading: false)}

      {:error, error} ->
        {:noreply,
         socket
         |> put_flash(:error, "Failed to load HSC lineage: #{inspect(error)}")
         |> assign(loading: false, lineage_data: nil)}
    end
  end

  def render(%{live_action: :sobol} = assigns) do
    ~H"""
    <CoreComponents.section title="Sobol Sensitivity Analysis">
      <CoreComponents.loading_container loading?={@loading}>
        <%= if @sobol_data do %>
          <.sobol_visualization data={@sobol_data} />
        <% else %>
          <p class="text-gray-500">No sensitivity data available.</p>
        <% end %>
      </CoreComponents.loading_container>
    </CoreComponents.section>
    """
  end

  def render(%{live_action: :hsc_lineage} = assigns) do
    ~H"""
    <CoreComponents.section title="HSC Lineage Tracking">
      <CoreComponents.loading_container loading?={@loading}>
        <%= if @lineage_data do %>
          <.lineage_visualization data={@lineage_data} />
        <% else %>
          <p class="text-gray-500">No lineage data available.</p>
        <% end %>
      </CoreComponents.loading_container>
    </CoreComponents.section>
    """
  end

  def render(%{entity_type: "parameters"} = assigns) do
    ~H"""
    <div class="space-y-6">
      <div class="md:flex md:items-center md:justify-between">
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate">
            All CDATA Parameters (32 parameters)
          </h2>
          <p class="mt-2 text-sm text-gray-600">
            Reduced model from 120 parameters via identifiability, sensitivity, and biological plausibility criteria
          </p>
        </div>
      </div>

      <CoreComponents.loading_container loading?={@loading}>
        <CoreComponents.table headers={["Parameter", "Symbol", "Value", "Units", "Status", "Source"]}>
          <tr :for={param <- @parameters} class="hover:bg-gray-50">
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
              <%= param["label"] %>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              <code class="bg-gray-100 px-2 py-1 rounded"><%= param["symbol"] %></code>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
              <%= param["value"] %>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              <%= param["unit"] %>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <CoreComponents.badge type={param["status_type"]}>
                <%= String.upcase(param["status"]) %>
              </CoreComponents.badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              <%= truncate_source(param["source"]) %>
            </td>
          </tr>
        </CoreComponents.table>
      </CoreComponents.loading_container>
    </div>
    """
  end

  def render(%{entity_type: "axiom"} = assigns) do
    ~H"""
    <div class="space-y-6">
      <CoreComponents.loading_container loading?={@loading}>
        <%= if @axiom do %>
          <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
            <div class="flex items-start">
              <div class="flex-shrink-0">
                <span class="inline-flex items-center justify-center h-12 w-12 rounded-full bg-yellow-100">
                  <span class="text-yellow-800 font-bold text-xl"><%= @axiom["number"] %></span>
                </span>
              </div>
              <div class="ml-4">
                <h3 class="text-2xl font-bold text-yellow-800">
                  Axiom <%= @axiom["number"] %>: <%= @axiom["title"] %>
                </h3>
                <div class="mt-4 prose prose-yellow max-w-none">
                  <p class="text-lg"><%= @axiom["mechanism"] %></p>
                </div>
                <div class="mt-6 bg-white border-l-4 border-yellow-400 p-4">
                  <h4 class="text-sm font-semibold text-gray-900">Key references:</h4>
                  <p class="mt-1 text-sm text-gray-600"><%= @axiom["references"] %></p>
                </div>
                <div class="mt-6 pt-6 border-t border-yellow-200">
                  <p class="text-sm text-yellow-700 italic">
                    This axiom must not be changed, revised, or removed without explicit user command. It must be present in all LOIs, grants, papers, and public CDATA materials.
                  </p>
                </div>
              </div>
            </div>
          </div>
        <% else %>
          <p class="text-gray-500">Axiom not found.</p>
        <% end %>
      </CoreComponents.loading_container>
    </div>
    """
  end

  defp sobol_visualization(assigns) do
    ~H"""
    <div class="space-y-6">
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">First-order Indices (Sᵢ)</h3>
          <div class="space-y-3">
            <div :for={item <- first_order(@data)} class="flex items-center justify-between">
              <span class="text-sm text-gray-600"><%= item.parameter %></span>
              <div class="flex items-center">
                <div class="w-32 bg-gray-200 rounded-full h-2.5 mr-3">
                  <div class="bg-blue-600 h-2.5 rounded-full" style={"width: #{Float.round(item.value * 100)}%"}></div>
                </div>
                <span class="text-sm font-medium text-gray-900"><%= Float.round(item.value, 3) %></span>
              </div>
            </div>
          </div>
        </div>

        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">Total-order Indices (Sₜᵢ)</h3>
          <div class="space-y-3">
            <div :for={item <- total_order(@data)} class="flex items-center justify-between">
              <span class="text-sm text-gray-600"><%= item.parameter %></span>
              <div class="flex items-center">
                <div class="w-32 bg-gray-200 rounded-full h-2.5 mr-3">
                  <div class="bg-purple-600 h-2.5 rounded-full" style={"width: #{Float.round(item.value * 100)}%"}></div>
                </div>
                <span class="text-sm font-medium text-gray-900"><%= Float.round(item.value, 3) %></span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">Interpretation</h3>
        <div class="prose max-w-none">
          <p>
            Sobol sensitivity analysis quantifies how much each parameter contributes to output variance.
            First-order indices (Sᵢ) measure direct effects; total-order indices (Sₜᵢ) include interactions.
          </p>
          <p class="mt-2">
            <strong>Key findings:</strong> Parameters with Sᵢ > 0.1 are primary drivers of model behavior.
            CDATA model reduction preserved parameters with highest total-order indices.
          </p>
        </div>
      </div>
    </div>
    """
  end

  defp lineage_visualization(assigns) do
    ~H"""
    <div class="space-y-6">
      <div class="bg-white border border-gray-200 rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">HSC Lineage Tree</h3>
        <div class="overflow-x-auto">
          <div class="inline-block min-w-full align-middle">
            <div class="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
              <table class="min-w-full divide-y divide-gray-300">
                <thead class="bg-gray-50">
                  <tr>
                    <th scope="col" class="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900">Generation</th>
                    <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Stem Daughters</th>
                    <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Differentiated</th>
                    <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">PTM Load</th>
                    <th scope="col" class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900">Asymmetry Index</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-200 bg-white">
                  <tr :for={row <- lineage_rows(@data)}>
                    <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900">
                      <%= row.generation %>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                      <%= row.stem_daughters %>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                      <%= row.differentiated %>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm">
                      <div class="flex items-center">
                        <div class="w-24 bg-gray-200 rounded-full h-2.5 mr-2">
                          <div class="bg-red-600 h-2.5 rounded-full" style={"width: #{row.ptm_percent}%"}></div>
                        </div>
                        <span><%= Float.round(row.ptm_load, 2) %></span>
                      </div>
                    </td>
                    <td class="whitespace-nowrap px-3 py-4 text-sm">
                      <div class="flex items-center">
                        <div class="w-24 bg-gray-200 rounded-full h-2.5 mr-2">
                          <div class="bg-green-600 h-2.5 rounded-full" style={"width: #{row.ai_percent}%"}></div>
                        </div>
                        <span><%= Float.round(row.asymmetry_index, 2) %></span>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">Lineage Statistics</h3>
          <dl class="grid grid-cols-1 gap-x-4 gap-y-6 sm:grid-cols-2">
            <div :for={stat <- lineage_stats(@data)} class="border-l-4 border-blue-500 pl-4">
              <dt class="text-sm font-medium text-gray-500"><%= stat.label %></dt>
              <dd class="mt-1 text-2xl font-semibold text-gray-900"><%= stat.value %></dd>
            </div>
          </dl>
        </div>

        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">Inheritance Pattern</h3>
          <div class="prose max-w-none">
            <p>
              Mother centriole inheritance follows asymmetric pattern with Ninein as molecular mediator.
              Inheritance ratio in HSC: <strong><%= inheritance_ratio(@data) %>%</strong> of stem daughters inherit old mother centriole.
            </p>
            <p class="mt-2">
              Critical prediction: AI = MFI(Ninein+)/MFI(Ninein−) = 2.1 (Royall 2023, Barandun 2025).
            </p>
          </div>
        </div>
      </div>
    </div>
    """
  end

  defp first_order(data) do
    Map.get(data, "first_order", [])
    |> Enum.sort_by(& &1["value"], :desc)
    |> Enum.map(&%{parameter: &1["parameter"], value: &1["value"]})
  end

  defp total_order(data) do
    Map.get(data, "total_order", [])
    |> Enum.sort_by(& &1["value"], :desc)
    |> Enum.map(&%{parameter: &1["parameter"], value: &1["value"]})
  end