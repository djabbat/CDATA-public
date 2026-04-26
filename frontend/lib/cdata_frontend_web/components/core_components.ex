defmodule CDATAFrontendWeb.CoreComponents do
  use Phoenix.Component

  attr :type, :string, default: nil
  attr :class, :string, default: nil
  attr :rest, :global

  slot :inner_block

  def badge(assigns) do
    ~H"""
    <span class={["inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium", badge_color(@type), @class]} {@rest}>
      <%= render_slot(@inner_block) %>
    </span>
    """
  end

  defp badge_color(type) do
    case type do
      "success" -> "bg-green-100 text-green-800"
      "warning" -> "bg-yellow-100 text-yellow-800"
      "error" -> "bg-red-100 text-red-800"
      "info" -> "bg-blue-100 text-blue-800"
      _ -> "bg-gray-100 text-gray-800"
    end
  end

  attr :id, :string, required: true
  attr :label, :string, required: true
  attr :value, :any, required: true
  attr :unit, :string, default: ""
  attr :description, :string, default: nil
  attr :status, :string, default: nil

  def parameter_card(assigns) do
    ~H"""
    <div class="bg-white overflow-hidden shadow rounded-lg border border-gray-200">
      <div class="px-4 py-5 sm:p-6">
        <dt class="text-sm font-medium text-gray-500 truncate">
          <%= @label %>
        </dt>
        <dd class="mt-1 text-3xl font-semibold text-gray-900">
          <%= @value %><span class="text-lg text-gray-500"> <%= @unit %></span>
        </dd>
        <%= if @description do %>
          <dd class="mt-2 text-sm text-gray-500">
            <%= @description %>
          </dd>
        <% end %>
        <%= if @status do %>
          <dd class="mt-2">
            <.badge type={status_type(@status)}>
              <%= String.upcase(@status) %>
            </.badge>
          </dd>
        <% end %>
      </div>
    </div>
    """
  end

  defp status_type(status) do
    case status do
      "measured" -> "success"
      "estimated" -> "warning"
      "tbd" -> "info"
      _ -> "info"
    end
  end

  attr :title, :string, required: true
  slot :inner_block, required: true

  def section(assigns) do
    ~H"""
    <div class="space-y-6">
      <div class="md:flex md:items-center md:justify-between">
        <div class="flex-1 min-w-0">
          <h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate">
            <%= @title %>
          </h2>
        </div>
      </div>
      <div class="bg-white shadow overflow-hidden sm:rounded-lg">
        <div class="px-4 py-5 sm:p-6">
          <%= render_slot(@inner_block) %>
        </div>
      </div>
    </div>
    """
  end

  attr :id, :string, required: true
  attr :title, :string, required: true
  attr :number, :integer, required: true
  slot :inner_block, required: true

  def axiom_card(assigns) do
    ~H"""
    <div class="bg-yellow-50 border-l-4 border-yellow-400 p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <span class="inline-flex items-center justify-center h-8 w-8 rounded-full bg-yellow-100">
            <span class="text-yellow-800 font-bold"><%= @number %></span>
          </span>
        </div>
        <div class="ml-3">
          <h3 class="text-lg font-medium text-yellow-800">
            <%= @title %>
          </h3>
          <div class="mt-2 text-sm text-yellow-700">
            <%= render_slot(@inner_block) %>
          </div>
        </div>
      </div>
    </div>
    """
  end

  attr :headers, :list, required: true
  slot :rows, required: true

  def table(assigns) do
    ~H"""
    <div class="flex flex-col">
      <div class="-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
          <div class="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
            <table class="min-w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th :for={header <- @headers} scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    <%= header %>
                  </th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                <%= render_slot(@rows) %>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
    """
  end

  attr :loading?, :boolean, default: false
  slot :inner_block

  def loading_container(assigns) do
    ~H"""
    <%= if @loading? do %>
      <div class="flex justify-center items-center h-64">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    <% else %>
      <%= render_slot(@inner_block) %>
    <% end %>
    """
  end
end