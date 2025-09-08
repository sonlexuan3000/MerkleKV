defmodule MerklekvClient.Exception do
  @moduledoc """
  Exception types for MerkleKV client operations.
  """
  
  defmodule ConnectionError do
    @moduledoc """
    Raised when connection to MerkleKV server fails.
    """
    defexception [:message, :reason]
    
    def exception(opts) when is_list(opts) do
      message = Keyword.get(opts, :message, "Connection failed")
      reason = Keyword.get(opts, :reason)
      %__MODULE__{message: message, reason: reason}
    end
    
    def exception(message) when is_binary(message) do
      %__MODULE__{message: message}
    end
  end
  
  defmodule TimeoutError do
    @moduledoc """
    Raised when an operation times out.
    """
    defexception [:message, :timeout]
    
    def exception(opts) when is_list(opts) do
      message = Keyword.get(opts, :message, "Operation timed out")
      timeout = Keyword.get(opts, :timeout)
      %__MODULE__{message: message, timeout: timeout}
    end
    
    def exception(message) when is_binary(message) do
      %__MODULE__{message: message}
    end
  end
  
  defmodule ProtocolError do
    @moduledoc """
    Raised when server returns an unexpected response.
    """
    defexception [:message, :response]
    
    def exception(opts) when is_list(opts) do
      message = Keyword.get(opts, :message, "Protocol error")
      response = Keyword.get(opts, :response)
      %__MODULE__{message: message, response: response}
    end
    
    def exception(message) when is_binary(message) do
      %__MODULE__{message: message}
    end
  end
  
  defmodule ValidationError do
    @moduledoc """
    Raised when client-side validation fails.
    """
    defexception [:message, :field, :value]
    
    def exception(opts) when is_list(opts) do
      message = Keyword.get(opts, :message, "Validation error")
      field = Keyword.get(opts, :field)
      value = Keyword.get(opts, :value)
      %__MODULE__{message: message, field: field, value: value}
    end
    
    def exception(message) when is_binary(message) do
      %__MODULE__{message: message}
    end
  end
  
  defmodule NetworkError do
    @moduledoc """
    Raised when network I/O operations fail.
    """
    defexception [:message, :reason]
    
    def exception(opts) when is_list(opts) do
      message = Keyword.get(opts, :message, "Network error")
      reason = Keyword.get(opts, :reason)
      %__MODULE__{message: message, reason: reason}
    end
    
    def exception(message) when is_binary(message) do
      %__MODULE__{message: message}
    end
  end
end
