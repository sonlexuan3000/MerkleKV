defmodule MerklekvClient do
  @moduledoc """
  Elixir client for MerkleKV distributed key-value store.
  
  This client provides a GenServer-based implementation with OTP supervision
  for fault-tolerant operations and connection management. It leverages Elixir's
  actor model for high concurrency and automatic error recovery.
  
  ## Features
  
  - **GenServer-based**: Built with OTP principles for fault tolerance
  - **Supervision**: Automatic restart and recovery on failures  
  - **Concurrent Operations**: Handle multiple requests simultaneously
  - **Connection Pooling**: Manage multiple connections efficiently
  - **Process Isolation**: Each client runs in its own process
  - **Timeout Handling**: Configurable timeouts for all operations
  
  ## Basic Usage
  
      # Start a client
      {:ok, client} = MerklekvClient.start_link(host: "localhost", port: 7379)
      
      # Basic operations
      :ok = MerklekvClient.set(client, "key", "value")
      {:ok, "value"} = MerklekvClient.get(client, "key")
      {:ok, true} = MerklekvClient.delete(client, "key")
      
      # Stop the client
      :ok = GenServer.stop(client)
  
  ## Supervision
  
      # Add to supervision tree
      children = [
        {MerklekvClient, [name: :my_client, host: "localhost", port: 7379]}
      ]
      
      Supervisor.start_link(children, strategy: :one_for_one)
      
      # Use the named client
      MerklekvClient.set(:my_client, "key", "value")
  
  ## Configuration
  
  The client accepts the following options:
  
  - `:host` - Server hostname (default: "localhost")
  - `:port` - Server port (default: 7379)
  - `:timeout` - Operation timeout in milliseconds (default: 5000)
  - `:max_retries` - Maximum retry attempts (default: 3)
  - `:reconnect_interval` - Reconnection interval in milliseconds (default: 1000)
  - `:name` - Process name for registration
  """
  
  use GenServer
  require Logger
  
  alias MerklekvClient.Exception
  
  @default_opts [
    host: "localhost",
    port: 7379,
    timeout: 5000,
    max_retries: 3,
    reconnect_interval: 1000
  ]
  
  @type option :: {:host, String.t()} 
                | {:port, pos_integer()} 
                | {:timeout, pos_integer()}
                | {:max_retries, non_neg_integer()}
                | {:reconnect_interval, pos_integer()}
                | {:name, atom()}
  
  @type t :: pid() | atom()
  
  # Client API
  
  @doc """
  Start a MerkleKV client process.
  
  ## Options
  
  #{Enum.map_join(@default_opts, "\n", fn {key, value} -> "  - `#{inspect(key)}` - Default: #{inspect(value)}" end)}
  - `:name` - Process name for registration
  
  ## Examples
  
      # Start with default options
      {:ok, client} = MerklekvClient.start_link()
      
      # Start with custom options
      {:ok, client} = MerklekvClient.start_link(
        host: "192.168.1.100", 
        port: 7379,
        timeout: 10_000,
        name: :my_client
      )
  """
  @spec start_link([option()]) :: GenServer.on_start()
  def start_link(opts \\ []) do
    {gen_opts, client_opts} = Keyword.split(opts, [:name])
    merged_opts = Keyword.merge(@default_opts, client_opts)
    GenServer.start_link(__MODULE__, merged_opts, gen_opts)
  end
  
  @doc """
  Get a value by key.
  
  ## Examples
  
      {:ok, "value"} = MerklekvClient.get(client, "key")
      {:error, :not_found} = MerklekvClient.get(client, "missing_key")
  """
  @spec get(t(), String.t()) :: {:ok, String.t()} | {:error, :not_found | term()}
  def get(client, key) when is_binary(key) do
    GenServer.call(client, {:get, key})
  end
  
  @doc """
  Set a key-value pair.
  
  ## Examples
  
      :ok = MerklekvClient.set(client, "key", "value")
  """
  @spec set(t(), String.t(), String.t()) :: :ok | {:error, term()}
  def set(client, key, value) when is_binary(key) and is_binary(value) do
    GenServer.call(client, {:set, key, value})
  end
  
  @doc """
  Delete a key.
  
  ## Examples
  
      {:ok, true} = MerklekvClient.delete(client, "existing_key")
      {:ok, false} = MerklekvClient.delete(client, "missing_key")
  """
  @spec delete(t(), String.t()) :: {:ok, boolean()} | {:error, term()}
  def delete(client, key) when is_binary(key) do
    GenServer.call(client, {:delete, key})
  end
  
  @doc """
  Get multiple keys at once.
  
  ## Examples
  
      {:ok, %{"key1" => "value1", "key2" => "value2"}} = 
        MerklekvClient.mget(client, ["key1", "key2", "missing_key"])
  """
  @spec mget(t(), [String.t()]) :: {:ok, map()} | {:error, term()}
  def mget(client, keys) when is_list(keys) do
    GenServer.call(client, {:mget, keys})
  end
  
  @doc """
  Set multiple key-value pairs.
  
  ## Examples
  
      :ok = MerklekvClient.mset(client, %{"key1" => "value1", "key2" => "value2"})
  """
  @spec mset(t(), map()) :: :ok | {:error, term()}
  def mset(client, pairs) when is_map(pairs) do
    GenServer.call(client, {:mset, pairs})
  end
  
  @doc """
  Delete multiple keys.
  
  ## Examples
  
      {:ok, %{"key1" => true, "key2" => false}} = 
        MerklekvClient.mdel(client, ["key1", "key2"])
  """
  @spec mdel(t(), [String.t()]) :: {:ok, map()} | {:error, term()}
  def mdel(client, keys) when is_list(keys) do
    GenServer.call(client, {:mdel, keys})
  end
  
  @doc """
  Check if the client is connected.
  
  ## Examples
  
      true = MerklekvClient.connected?(client)
  """
  @spec connected?(t()) :: boolean()
  def connected?(client) do
    GenServer.call(client, :connected?)
  end
  
  @doc """
  Get client configuration.
  
  ## Examples
  
      config = MerklekvClient.config(client)
      #=> [host: "localhost", port: 7379, timeout: 5000, ...]
  """
  @spec config(t()) :: keyword()
  def config(client) do
    GenServer.call(client, :config)
  end
  
  @doc """
  Execute a function with automatic connection management.
  
  ## Examples
  
      {:ok, "value"} = MerklekvClient.with_connection([host: "localhost"], fn client ->
        MerklekvClient.set(client, "key", "value")
        MerklekvClient.get(client, "key")
      end)
  """
  @spec with_connection([option()], (t() -> term())) :: {:ok, term()} | {:error, term()}
  def with_connection(opts \\ [], fun) when is_function(fun, 1) do
    case start_link(opts) do
      {:ok, client} ->
        try do
          result = fun.(client)
          {:ok, result}
        after
          GenServer.stop(client)
        end
      error ->
        error
    end
  end
  
  @doc """
  Execute multiple operations in a pipeline (single network round-trip).
  
  ## Examples
  
      {:ok, [nil, :ok, {:ok, true}]} = MerklekvClient.pipeline(client, [
        {:get, "key1"},
        {:set, "key2", "value2"}, 
        {:delete, "key3"}
      ])
  """
  @spec pipeline(t(), [{atom(), String.t()} | {atom(), String.t(), String.t()}]) :: {:ok, [term()]} | {:error, term()}
  def pipeline(client, operations) when is_list(operations) do
    GenServer.call(client, {:pipeline, operations})
  end
  
  @doc """
  Health check operation.
  
  ## Examples
  
      {:ok, true} = MerklekvClient.health_check(client)
  """
  @spec health_check(t()) :: {:ok, boolean()} | {:error, term()}
  def health_check(client) do
    case get(client, "__health__") do
      {:ok, _} -> {:ok, true}
      {:error, :not_found} -> {:ok, true}
      {:error, _} -> {:ok, false}
    end
  end
  
  # GenServer callbacks
  
  @impl true
  def init(opts) do
    state = %{
      host: Keyword.fetch!(opts, :host),
      port: Keyword.fetch!(opts, :port),
      timeout: Keyword.fetch!(opts, :timeout),
      max_retries: Keyword.fetch!(opts, :max_retries),
      reconnect_interval: Keyword.fetch!(opts, :reconnect_interval),
      socket: nil,
      connected: false
    }
    
    # Connect asynchronously
    send(self(), :connect)
    
    {:ok, state}
  end
  
  @impl true
  def handle_call({:get, key}, _from, state) do
    with :ok <- validate_key(key),
         {:ok, socket} <- ensure_connected(state) do
      case send_command(socket, "GET #{key}", state.timeout) do
        {:ok, "NOT_FOUND"} ->
          {:reply, {:error, :not_found}, state}
        {:ok, "VALUE " <> value} ->
          {:reply, {:ok, value}, state}
        {:ok, "VALUE \"\"" <> _} ->
          {:reply, {:ok, ""}, state}
        {:ok, other} ->
          {:reply, {:error, Exception.ProtocolError.exception(response: other)}, state}
        {:error, reason} ->
          {:reply, {:error, reason}, %{state | connected: false, socket: nil}}
      end
    else
      error -> {:reply, error, state}
    end
  end
  
  @impl true
  def handle_call({:set, key, value}, _from, state) do
    with :ok <- validate_key(key),
         :ok <- validate_value(value),
         {:ok, socket} <- ensure_connected(state) do
      case send_command(socket, "SET #{key} #{value}", state.timeout) do
        {:ok, "OK"} ->
          {:reply, :ok, state}
        {:ok, other} ->
          {:reply, {:error, Exception.ProtocolError.exception(response: other)}, state}
        {:error, reason} ->
          {:reply, {:error, reason}, %{state | connected: false, socket: nil}}
      end
    else
      error -> {:reply, error, state}
    end
  end
  
  @impl true
  def handle_call({:delete, key}, _from, state) do
    with :ok <- validate_key(key),
         {:ok, socket} <- ensure_connected(state) do
      case send_command(socket, "DEL #{key}", state.timeout) do
        {:ok, "DELETED"} ->
          {:reply, {:ok, true}, state}
        {:ok, "NOT_FOUND"} ->
          {:reply, {:ok, false}, state}
        {:ok, other} ->
          {:reply, {:error, Exception.ProtocolError.exception(response: other)}, state}
        {:error, reason} ->
          {:reply, {:error, reason}, %{state | connected: false, socket: nil}}
      end
    else
      error -> {:reply, error, state}
    end
  end
  
  @impl true
  def handle_call({:pipeline, operations}, _from, state) do
    with {:ok, socket} <- ensure_connected(state) do
      # Build all commands
      commands = 
        Enum.map(operations, fn
          {:get, key} -> 
            with :ok <- validate_key(key), do: "GET #{key}"
          {:set, key, value} -> 
            with :ok <- validate_key(key), :ok <- validate_value(value), do: "SET #{key} #{value}"
          {:delete, key} -> 
            with :ok <- validate_key(key), do: "DEL #{key}"
        end)
      
      case Enum.find(commands, fn cmd -> is_tuple(cmd) and elem(cmd, 0) == :error end) do
        nil ->
          # Send all commands in pipeline
          pipeline_command = Enum.join(commands, "\r\n") <> "\r\n"
          case :gen_tcp.send(socket, pipeline_command) do
            :ok ->
              # Read all responses
              results = 
                Enum.map(operations, fn operation ->
                  receive do
                    {:tcp, ^socket, data} ->
                      response = String.trim(data)
                      case operation do
                        {:get, _} ->
                          case response do
                            "NOT_FOUND" -> {:error, :not_found}
                            "VALUE " <> value -> {:ok, value}
                            "VALUE \"\"" -> {:ok, ""}
                            other -> {:error, Exception.ProtocolError.exception(response: other)}
                          end
                        {:set, _, _} ->
                          case response do
                            "OK" -> :ok
                            other -> {:error, Exception.ProtocolError.exception(response: other)}
                          end
                        {:delete, _} ->
                          case response do
                            "DELETED" -> {:ok, true}
                            "NOT_FOUND" -> {:ok, false}
                            other -> {:error, Exception.ProtocolError.exception(response: other)}
                          end
                      end
                    after
                      state.timeout -> {:error, Exception.TimeoutError.exception(timeout: state.timeout)}
                  end
                end)
              
              {:reply, {:ok, results}, state}
            {:error, reason} ->
              {:reply, {:error, Exception.NetworkError.exception(reason: reason)}, %{state | connected: false, socket: nil}}
          end
        error ->
          {:reply, error, state}
      end
    else
      error -> {:reply, error, state}
    end
  end
  
  @impl true
  def handle_call({:mget, keys}, _from, state) do
    results = 
      Enum.reduce_while(keys, {:ok, %{}}, fn key, {:ok, acc} ->
        case get_single(key, state) do
          {:ok, value} -> {:cont, {:ok, Map.put(acc, key, value)}}
          {:error, :not_found} -> {:cont, {:ok, acc}}
          {:error, reason} -> {:halt, {:error, reason}}
        end
      end)
    
    {:reply, results, state}
  end
  
  @impl true
  def handle_call({:mset, pairs}, _from, state) do
    result = 
      Enum.reduce_while(pairs, :ok, fn {key, value}, :ok ->
        case set_single(key, value, state) do
          :ok -> {:cont, :ok}
          {:error, reason} -> {:halt, {:error, reason}}
        end
      end)
    
    {:reply, result, state}
  end
  
  @impl true
  def handle_call({:mdel, keys}, _from, state) do
    results = 
      Enum.reduce_while(keys, {:ok, %{}}, fn key, {:ok, acc} ->
        case delete_single(key, state) do
          {:ok, deleted} -> {:cont, {:ok, Map.put(acc, key, deleted)}}
          {:error, reason} -> {:halt, {:error, reason}}
        end
      end)
    
    {:reply, results, state}
  end
  
  @impl true
  def handle_call(:connected?, _from, state) do
    {:reply, state.connected, state}
  end
  
  @impl true
  def handle_call(:config, _from, state) do
    config = [
      host: state.host,
      port: state.port,
      timeout: state.timeout,
      max_retries: state.max_retries,
      reconnect_interval: state.reconnect_interval
    ]
    {:reply, config, state}
  end
  
  @impl true
  def handle_info(:connect, state) do
    case connect(state) do
      {:ok, socket} ->
        Logger.info("Connected to MerkleKV server at #{state.host}:#{state.port}")
        {:noreply, %{state | socket: socket, connected: true}}
      {:error, reason} ->
        Logger.warning("Failed to connect to MerkleKV server: #{inspect(reason)}")
        Process.send_after(self(), :connect, state.reconnect_interval)
        {:noreply, state}
    end
  end
  
  @impl true
  def handle_info({:tcp_closed, _socket}, state) do
    Logger.warning("Connection to MerkleKV server closed")
    send(self(), :connect)
    {:noreply, %{state | socket: nil, connected: false}}
  end
  
  @impl true
  def handle_info({:tcp_error, _socket, reason}, state) do
    Logger.error("TCP error: #{inspect(reason)}")
    send(self(), :connect)
    {:noreply, %{state | socket: nil, connected: false}}
  end
  
  @impl true
  def terminate(_reason, state) do
    if state.socket do
      :gen_tcp.close(state.socket)
    end
    :ok
  end
  
  # Private functions
  
  defp connect(state) do
    case :gen_tcp.connect(
      to_charlist(state.host), 
      state.port, 
      [:binary, packet: :line, active: true, :nodelay],  # Enable TCP_NODELAY
      state.timeout
    ) do
      {:ok, socket} -> {:ok, socket}
      {:error, reason} -> {:error, Exception.ConnectionError.exception(reason: reason)}
    end
  end
  
  defp ensure_connected(%{connected: true, socket: socket} = _state) when socket != nil do
    {:ok, socket}
  end
  
  defp ensure_connected(_state) do
    {:error, Exception.ConnectionError.exception("Not connected to server")}
  end
  
  defp send_command(socket, command, timeout) do
    case :gen_tcp.send(socket, command <> "\r\n") do
      :ok ->
        receive do
          {:tcp, ^socket, data} ->
            response = String.trim(data)
            {:ok, response}
          {:tcp_closed, ^socket} ->
            {:error, Exception.NetworkError.exception("Connection closed")}
          {:tcp_error, ^socket, reason} ->
            {:error, Exception.NetworkError.exception(reason: reason)}
        after
          timeout ->
            {:error, Exception.TimeoutError.exception(timeout: timeout)}
        end
      {:error, reason} ->
        {:error, Exception.NetworkError.exception(reason: reason)}
    end
  end
  
  defp get_single(key, state) do
    with :ok <- validate_key(key),
         {:ok, socket} <- ensure_connected(state) do
      case send_command(socket, "GET #{key}", state.timeout) do
        {:ok, "NOT_FOUND"} -> {:error, :not_found}
        {:ok, "VALUE " <> value} -> {:ok, value}
        {:ok, "VALUE \"\"" <> _} -> {:ok, ""}
        {:ok, other} -> {:error, Exception.ProtocolError.exception(response: other)}
        error -> error
      end
    end
  end
  
  defp set_single(key, value, state) do
    with :ok <- validate_key(key),
         :ok <- validate_value(value),
         {:ok, socket} <- ensure_connected(state) do
      case send_command(socket, "SET #{key} #{value}", state.timeout) do
        {:ok, "OK"} -> :ok
        {:ok, other} -> {:error, Exception.ProtocolError.exception(response: other)}
        error -> error
      end
    end
  end
  
  defp delete_single(key, state) do
    with :ok <- validate_key(key),
         {:ok, socket} <- ensure_connected(state) do
      case send_command(socket, "DEL #{key}", state.timeout) do
        {:ok, "DELETED"} -> {:ok, true}
        {:ok, "NOT_FOUND"} -> {:ok, false}
        {:ok, other} -> {:error, Exception.ProtocolError.exception(response: other)}
        error -> error
      end
    end
  end
  
  defp validate_key("") do
    {:error, Exception.ValidationError.exception(message: "Key cannot be empty", field: :key)}
  end
  
  defp validate_key(key) when is_binary(key) do
    if String.contains?(key, ["\n", "\r"]) do
      {:error, Exception.ValidationError.exception(
        message: "Key cannot contain newlines", 
        field: :key, 
        value: key
      )}
    else
      :ok
    end
  end
  
  defp validate_value(value) when is_binary(value) do
    if String.contains?(value, ["\n", "\r"]) do
      {:error, Exception.ValidationError.exception(
        message: "Value cannot contain newlines", 
        field: :value, 
        value: value
      )}
    else
      :ok
    end
  end
end
