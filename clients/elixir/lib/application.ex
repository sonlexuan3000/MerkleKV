defmodule MerklekvClient.Application do
  @moduledoc """
  Application module for MerkleKV client library.
  
  Sets up the supervision tree for managing client connections.
  """
  
  use Application
  
  @impl true
  def start(_type, _args) do
    children = [
      {DynamicSupervisor, strategy: :one_for_one, name: MerklekvClient.ClientSupervisor}
    ]
    
    opts = [strategy: :one_for_one, name: MerklekvClient.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
