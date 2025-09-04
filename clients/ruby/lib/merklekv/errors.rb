module MerkleKV
  # Base error class for all MerkleKV client errors
  class Error < StandardError; end

  # Error raised when connection-related issues occur
  class ConnectionError < Error; end

  # Error raised when operation times out
  class TimeoutError < Error; end

  # Error raised when protocol errors occur (ERROR response from server)
  class ProtocolError < Error; end
end
