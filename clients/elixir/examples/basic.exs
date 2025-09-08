defmodule MerklekvClient.Examples.Basic do
  @moduledoc """
  Basic example demonstrating MerkleKV Elixir client usage.
  
  Run with: `mix run examples/basic.exs`
  """
  
  require Logger
  
  def run do
    IO.puts("🚀 MerkleKV Elixir Client Example")
    IO.puts("Starting MerkleKV client...")
    
    case MerklekvClient.start_link(host: "localhost", port: 7379) do
      {:ok, client} ->
        # Wait for connection
        :timer.sleep(200)
        
        if MerklekvClient.connected?(client) do
          IO.puts("✅ Connected successfully!")
          
          try do
            run_examples(client)
            IO.puts("\n✅ Example completed successfully!")
          catch
            kind, reason ->
              IO.puts("\n❌ Error: #{inspect({kind, reason})}")
          after
            GenServer.stop(client)
          end
        else
          IO.puts("❌ Failed to connect to MerkleKV server")
          IO.puts("Make sure MerkleKV server is running on localhost:7379")
        end
        
      {:error, reason} ->
        IO.puts("❌ Failed to start client: #{inspect(reason)}")
    end
  end
  
  defp run_examples(client) do
    basic_operations(client)
    unicode_support(client)
    empty_values(client)
    large_values(client)
    multi_key_operations(client)
    error_handling_examples(client)
    cleanup(client)
  end
  
  defp basic_operations(client) do
    IO.puts("\n📝 Basic Operations:")
    
    # Set operation
    :ok = MerklekvClient.set(client, "elixir:example", "Hello from Elixir!")
    IO.puts("SET elixir:example = 'Hello from Elixir!'")
    
    # Get operation
    {:ok, value} = MerklekvClient.get(client, "elixir:example")
    IO.puts("GET elixir:example = '#{value}'")
    
    # Update value
    :ok = MerklekvClient.set(client, "elixir:example", "Updated value")
    {:ok, updated_value} = MerklekvClient.get(client, "elixir:example")
    IO.puts("GET elixir:example = '#{updated_value}' (after update)")
    
    # Delete operation
    {:ok, deleted} = MerklekvClient.delete(client, "elixir:example")
    IO.puts("DELETE elixir:example = #{deleted}")
    
    # Verify deletion
    case MerklekvClient.get(client, "elixir:example") do
      {:error, :not_found} -> IO.puts("GET elixir:example = nil (after deletion)")
      {:ok, value} -> IO.puts("GET elixir:example = '#{value}' (unexpected!)")
    end
  end
  
  defp unicode_support(client) do
    IO.puts("\n🌍 Unicode Support:")
    unicode_value = "Hello 世界! 🚀 Café Ñoël"
    :ok = MerklekvClient.set(client, "unicode", unicode_value)
    {:ok, retrieved_value} = MerklekvClient.get(client, "unicode")
    IO.puts("Unicode value: '#{retrieved_value}'")
  end
  
  defp empty_values(client) do
    IO.puts("\n📭 Empty Values:")
    :ok = MerklekvClient.set(client, "empty", "")
    {:ok, empty_value} = MerklekvClient.get(client, "empty")
    IO.puts("Empty value: '#{empty_value}' (length: #{String.length(empty_value)})")
  end
  
  defp large_values(client) do
    IO.puts("\n📊 Large Values:")
    large_value = String.duplicate("Elixir", 1000)
    :ok = MerklekvClient.set(client, "large", large_value)
    {:ok, retrieved_large} = MerklekvClient.get(client, "large")
    IO.puts("Large value stored and retrieved (length: #{String.length(retrieved_large)})")
  end
  
  defp multi_key_operations(client) do
    IO.puts("\n📦 Multi-Key Operations:")
    
    # Multi-set
    pairs = %{
      "multi:1" => "value1",
      "multi:2" => "value2", 
      "multi:3" => "value3"
    }
    :ok = MerklekvClient.mset(client, pairs)
    IO.puts("MSET: Set #{map_size(pairs)} key-value pairs")
    
    # Multi-get
    {:ok, mget_result} = MerklekvClient.mget(client, ["multi:1", "multi:2", "multi:3", "multi:missing"])
    IO.puts("MGET result: #{inspect(mget_result)}")
    
    # Multi-delete
    {:ok, mdel_result} = MerklekvClient.mdel(client, ["multi:1", "multi:2", "multi:3"])
    IO.puts("MDEL result: #{inspect(mdel_result)}")
  end
  
  defp error_handling_examples(client) do
    IO.puts("\n🛡️ Error Handling Examples:")
    
    # Empty key validation
    case MerklekvClient.set(client, "", "value") do
      {:error, %MerklekvClient.Exception.ValidationError{}} ->
        IO.puts("✅ Caught validation error for empty key")
      other ->
        IO.puts("❌ Unexpected result for empty key: #{inspect(other)}")
    end
    
    # Key with newlines validation
    case MerklekvClient.set(client, "key\nwith\nnewlines", "value") do
      {:error, %MerklekvClient.Exception.ValidationError{}} ->
        IO.puts("✅ Caught validation error for key with newlines")
      other ->
        IO.puts("❌ Unexpected result for key with newlines: #{inspect(other)}")
    end
    
    # Value with newlines validation
    case MerklekvClient.set(client, "key", "value\nwith\nnewlines") do
      {:error, %MerklekvClient.Exception.ValidationError{}} ->
        IO.puts("✅ Caught validation error for value with newlines")
      other ->
        IO.puts("❌ Unexpected result for value with newlines: #{inspect(other)}")
    end
  end
  
  defp cleanup(client) do
    # Clean up test data
    MerklekvClient.delete(client, "unicode")
    MerklekvClient.delete(client, "empty")
    MerklekvClient.delete(client, "large")
  end
end

defmodule MerklekvClient.Examples.Supervision do
  @moduledoc """
  Example demonstrating supervision and fault tolerance.
  """
  
  require Logger
  
  def run do
    IO.puts("\n👥 Supervision Example:")
    
    # Define child spec for supervised client
    children = [
      {MerklekvClient, [name: :supervised_client, host: "localhost", port: 7379]}
    ]
    
    case Supervisor.start_link(children, strategy: :one_for_one, name: :example_supervisor) do
      {:ok, supervisor} ->
        # Wait for connection
        :timer.sleep(300)
        
        if MerklekvClient.connected?(:supervised_client) do
          IO.puts("✅ Supervised client connected!")
          
          # Test operations with supervised client
          :ok = MerklekvClient.set(:supervised_client, "supervised:test", "supervised value")
          {:ok, value} = MerklekvClient.get(:supervised_client, "supervised:test")
          IO.puts("Supervised client test: '#{value}'")
          
          # Cleanup
          MerklekvClient.delete(:supervised_client, "supervised:test")
          
          IO.puts("✅ Supervision example completed!")
        else
          IO.puts("❌ Supervised client failed to connect")
        end
        
        Supervisor.stop(supervisor)
        
      {:error, reason} ->
        IO.puts("❌ Failed to start supervisor: #{inspect(reason)}")
    end
  end
end

defmodule MerklekvClient.Examples.Concurrency do
  @moduledoc """
  Example demonstrating concurrent operations.
  """
  
  require Logger
  
  def run do
    IO.puts("\n🔄 Concurrency Example:")
    
    case MerklekvClient.start_link(host: "localhost", port: 7379) do
      {:ok, client} ->
        # Wait for connection
        :timer.sleep(200)
        
        if MerklekvClient.connected?(client) do
          IO.puts("Starting concurrent operations...")
          
          # Launch concurrent tasks
          tasks = 
            for i <- 1..20 do
              Task.async(fn ->
                key = "concurrent:#{i}"
                value = "value:#{i}"
                
                # Set the value
                :ok = MerklekvClient.set(client, key, value)
                
                # Get and verify the value
                {:ok, ^value} = MerklekvClient.get(client, key)
                
                # Delete the key
                {:ok, true} = MerklekvClient.delete(client, key)
                
                IO.puts("Process #{i}: Completed operations for #{key}")
                i
              end)
            end
          
          # Wait for all tasks to complete
          results = Task.await_many(tasks, 10_000)
          
          IO.puts("✅ All #{length(results)} concurrent operations completed!")
        else
          IO.puts("❌ Failed to connect to server")
        end
        
        GenServer.stop(client)
        
      {:error, reason} ->
        IO.puts("❌ Failed to start client: #{inspect(reason)}")
    end
  end
end

defmodule MerklekvClient.Examples.WithConnection do
  @moduledoc """
  Example demonstrating the with_connection convenience method.
  """
  
  def run do
    IO.puts("\n🔧 With Connection Example:")
    
    case MerklekvClient.with_connection([host: "localhost", port: 7379], fn client ->
      # Wait for connection
      :timer.sleep(200)
      
      # Perform operations
      :ok = MerklekvClient.set(client, "convenience:test", "convenience value")
      {:ok, value} = MerklekvClient.get(client, "convenience:test")
      MerklekvClient.delete(client, "convenience:test")
      
      value
    end) do
      {:ok, result} ->
        IO.puts("✅ Convenience method result: '#{result}'")
        
      {:error, reason} ->
        IO.puts("❌ Convenience method error: #{inspect(reason)}")
    end
  end
end

# Main execution
if Mix.env() != :test do
  MerklekvClient.Examples.Basic.run()
  MerklekvClient.Examples.Supervision.run()
  MerklekvClient.Examples.Concurrency.run()
  MerklekvClient.Examples.WithConnection.run()
end
