require 'socket'
require 'timeout'
require_relative 'errors'

module MerkleKV
  # Official Ruby client for MerkleKV distributed key-value store.
  #
  # This client implements the TCP text protocol with CRLF termination and UTF-8 encoding.
  # Empty values in SET operations are automatically represented as "" at the protocol layer.
  #
  # @example Basic usage
  #   client = MerkleKV::Client.new(host: "127.0.0.1", port: 7379, timeout: 5.0)
  #   client.set("user:1", "alice")
  #   value = client.get("user:1")     # => "alice"
  #   deleted = client.delete("user:1") # => true
  #   client.close
  class Client
    # Default connection timeout in seconds
    DEFAULT_TIMEOUT = 5.0

    # Initialize a new MerkleKV client.
    #
    # @param host [String] Server hostname or IP address
    # @param port [Integer] Server port
    # @param timeout [Float] Operation timeout in seconds
    # @example
    #   client = MerkleKV::Client.new(host: "127.0.0.1", port: 7379, timeout: 10.0)
    def initialize(host: "127.0.0.1", port: 7379, timeout: DEFAULT_TIMEOUT)
      @host = host
      @port = port
      @timeout = timeout
      @socket = nil
    end

    # Set a key-value pair.
    # Empty values are automatically handled - they are represented as "" at the protocol layer.
    #
    # @param key [String] Key to set (cannot be empty)
    # @param value [String] Value to set (empty values are automatically converted to "")
    # @raise [ArgumentError] if key is empty or nil
    # @raise [ConnectionError] if connection fails
    # @raise [TimeoutError] if operation times out
    # @raise [ProtocolError] if server returns an error
    # @example
    #   client.set("user:123", "john_doe")
    #   client.set("empty:key", "")  # Automatically handled
    def set(key, value)
      validate_key(key)
      raise ArgumentError, "Value cannot be nil" if value.nil?

      formatted_value = format_value(value)
      command = "SET #{key} #{formatted_value}"
      
      response = send_command(command)
      
      return if response == "OK"
      
      if response.start_with?("ERROR ")
        raise ProtocolError, response[6..-1]
      end
      
      raise ProtocolError, "Unexpected response: #{response}"
    end

    # Get a value by key.
    #
    # @param key [String] Key to retrieve (cannot be empty)
    # @return [String, nil] Value if found, nil if key not found
    # @raise [ArgumentError] if key is empty or nil
    # @raise [ConnectionError] if connection fails
    # @raise [TimeoutError] if operation times out
    # @raise [ProtocolError] if server returns an error
    # @example
    #   value = client.get("user:123")  # => "john_doe" or nil
    def get(key)
      validate_key(key)
      
      command = "GET #{key}"
      response = send_command(command)
      
      return nil if response == "(null)" || response == "NOT_FOUND"
      
      if response.start_with?("ERROR ")
        raise ProtocolError, response[6..-1]
      end
      
      # Strip "VALUE " prefix from server response
      if response.start_with?("VALUE ")
        value = response[6..-1]
        # Handle empty value represented as ""
        return "" if value == '""'
        return value
      end
      
      response
    end

    # Delete a key.
    #
    # @param key [String] Key to delete (cannot be empty)
    # @return [Boolean] true if key was deleted, false if key was not found
    # @raise [ArgumentError] if key is empty or nil
    # @raise [ConnectionError] if connection fails
    # @raise [TimeoutError] if operation times out
    # @raise [ProtocolError] if server returns an error
    # @example
    #   deleted = client.delete("user:123")  # => true or false
    def delete(key)
      validate_key(key)
      
      command = "DEL #{key}"
      response = send_command(command)
      
      case response
      when "OK"
        true  # Server returns OK for both existing and non-existing keys
      when "NOT_FOUND"
        false
      else
        if response.start_with?("ERROR ")
          raise ProtocolError, response[6..-1]
        end
        raise ProtocolError, "Unexpected response: #{response}"
      end
    end

    # Close the connection to the server.
    # This method is idempotent and can be called multiple times safely.
    #
    # @example
    #   client.close
    def close
      return unless @socket
      
      begin
        @socket.close unless @socket.closed?
      rescue StandardError
        # Ignore errors during close
      ensure
        @socket = nil
      end
    end

    # Check if the client is connected.
    #
    # @return [Boolean] true if connected, false otherwise
    def connected?
      @socket && !@socket.closed? ? true : false
    end

    private

    # Ensure connection is established
    def ensure_connected
      return if connected?

      begin
        Timeout.timeout(@timeout) do
          @socket = TCPSocket.new(@host, @port)
          @socket.setsockopt(Socket::IPPROTO_TCP, Socket::TCP_NODELAY, 1)
        end
      rescue Timeout::Error
        raise TimeoutError, "Connection timeout to #{@host}:#{@port}"
      rescue SocketError, Errno::ECONNREFUSED, Errno::EHOSTUNREACH => e
        raise ConnectionError, "Failed to connect to #{@host}:#{@port}: #{e.message}"
      rescue StandardError => e
        raise ConnectionError, "Connection error: #{e.message}"
      end
    end

    # Send a command and return the response
    def send_command(command)
      ensure_connected
      
      begin
        Timeout.timeout(@timeout) do
          # Send command with CRLF termination
          @socket.puts(command)
          
          # Read response until CRLF
          response = @socket.gets
          raise ConnectionError, "Server closed connection" if response.nil?
          
          # Remove CRLF terminator and ensure UTF-8 encoding
          response.chomp.force_encoding('UTF-8')
        end
      rescue Timeout::Error
        close
        raise TimeoutError, "Operation timeout"
      rescue IOError, Errno::EPIPE, Errno::ECONNRESET => e
        close
        raise ConnectionError, "Network I/O error: #{e.message}"
      rescue StandardError => e
        close
        raise ConnectionError, "Communication error: #{e.message}"
      end
    end

    # Format a value for the SET command. Empty strings are represented as "".
    def format_value(value)
      value.empty? ? '""' : value
    end

    # Validate that key is not empty or nil
    def validate_key(key)
      raise ArgumentError, "Key cannot be nil" if key.nil?
      raise ArgumentError, "Key cannot be empty" if key.empty?
    end
  end
end
