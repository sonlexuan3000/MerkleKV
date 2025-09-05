using System;
using System.IO;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace MerkleKV;

/// <summary>
/// Official .NET client for MerkleKV distributed key-value store.
/// Implements the TCP text protocol with CRLF termination and UTF-8 encoding.
/// </summary>
public sealed class MerkleKvClient : IDisposable, IAsyncDisposable
{
    private readonly string _host;
    private readonly int _port;
    private readonly TimeSpan _timeout;
    private TcpClient? _tcpClient;
    private NetworkStream? _stream;
    private StreamReader? _reader;
    private StreamWriter? _writer;
    private bool _disposed;

    /// <summary>
    /// Initializes a new MerkleKV client.
    /// </summary>
    /// <param name="host">Server hostname or IP address</param>
    /// <param name="port">Server port</param>
    /// <param name="timeout">Operation timeout (default: 5 seconds)</param>
    public MerkleKvClient(string host = "127.0.0.1", int port = 7379, TimeSpan? timeout = null)
    {
        _host = host ?? throw new ArgumentNullException(nameof(host));
        _port = port;
        _timeout = timeout ?? TimeSpan.FromSeconds(5);
    }

    /// <summary>
    /// Ensures the client is connected to the server.
    /// </summary>
    private async Task EnsureConnectedAsync(CancellationToken cancellationToken = default)
    {
        if (_tcpClient?.Connected == true)
            return;

        try
        {
            _tcpClient?.Close();
            _tcpClient = new TcpClient();
            _tcpClient.ReceiveTimeout = (int)_timeout.TotalMilliseconds;
            _tcpClient.SendTimeout = (int)_timeout.TotalMilliseconds;

            using var cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            cts.CancelAfter(_timeout);

            await _tcpClient.ConnectAsync(_host, _port, cts.Token);
            
            _stream = _tcpClient.GetStream();
            _reader = new StreamReader(_stream, Encoding.UTF8, leaveOpen: true);
            _writer = new StreamWriter(_stream, Encoding.UTF8, leaveOpen: true) { NewLine = "\r\n" };
        }
        catch (SocketException ex)
        {
            throw new MerkleKvConnectionException($"Failed to connect to {_host}:{_port}", ex);
        }
        catch (OperationCanceledException ex) when (ex.CancellationToken.IsCancellationRequested)
        {
            throw new MerkleKvTimeoutException($"Connection timeout to {_host}:{_port}", ex);
        }
    }

    /// <summary>
    /// Sends a command and returns the response.
    /// </summary>
    private async Task<string> SendCommandAsync(string command, CancellationToken cancellationToken = default)
    {
        if (_disposed) throw new ObjectDisposedException(nameof(MerkleKvClient));
        
        const int maxRetries = 3;
        
        for (int attempt = 0; attempt < maxRetries; attempt++)
        {
            try
            {
                await EnsureConnectedAsync(cancellationToken);

                await _writer!.WriteLineAsync(command);
                await _writer.FlushAsync();

                using var cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
                cts.CancelAfter(_timeout);
                
                var response = await _reader!.ReadLineAsync();
                if (response == null)
                    throw new MerkleKvConnectionException("Server closed connection unexpectedly");

                // Check for server corruption and retry if needed
                if (response.StartsWith("Unknown command:") && attempt < maxRetries - 1)
                {
                    // Server state corruption - disconnect and retry
                    await DisconnectAsync();
                    continue;
                }

                return response;
            }
            catch (IOException ex) when (attempt < maxRetries - 1)
            {
                // Retry on I/O errors
                await DisconnectAsync();
                continue;
            }
            catch (IOException ex)
            {
                throw new MerkleKvConnectionException("Network I/O error", ex);
            }
            catch (OperationCanceledException ex) when (ex.CancellationToken.IsCancellationRequested)
            {
                throw new MerkleKvTimeoutException("Operation timeout", ex);
            }
        }
        
        throw new MerkleKvConnectionException($"Failed to execute command after {maxRetries} attempts");
    }

    /// <summary>
    /// Formats a value for the SET command. Empty strings are represented as "".
    /// </summary>
    private static string FormatValue(string value)
    {
        return string.IsNullOrEmpty(value) ? "\"\"" : value;
    }

    /// <summary>
    /// Sets a key-value pair.
    /// </summary>
    /// <param name="key">Key to set</param>
    /// <param name="value">Value to set (empty values are automatically handled)</param>
    public void Set(string key, string value)
    {
        SetAsync(key, value, CancellationToken.None).GetAwaiter().GetResult();
    }

    /// <summary>
    /// Sets a key-value pair asynchronously.
    /// </summary>
    /// <param name="key">Key to set</param>
    /// <param name="value">Value to set (empty values are automatically handled)</param>
    /// <param name="cancellationToken">Cancellation token</param>
    public async Task SetAsync(string key, string value, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrEmpty(key))
            throw new ArgumentException("Key cannot be null or empty", nameof(key));
        if (value == null)
            throw new ArgumentNullException(nameof(value));

        var formattedValue = FormatValue(value);
        var command = $"SET {key} {formattedValue}";
        var response = await SendCommandAsync(command, cancellationToken);

        if (response == "OK")
            return;

        if (response.StartsWith("ERROR "))
            throw new MerkleKvProtocolException(response.Substring(6));

        throw new MerkleKvProtocolException($"Unexpected response: {response}");
    }

    /// <summary>
    /// Gets a value by key.
    /// </summary>
    /// <param name="key">Key to retrieve</param>
    /// <returns>Value or null if key not found</returns>
    public string? Get(string key)
    {
        return GetAsync(key, CancellationToken.None).GetAwaiter().GetResult();
    }

    /// <summary>
    /// Gets a value by key asynchronously.
    /// </summary>
    /// <param name="key">Key to retrieve</param>
    /// <param name="cancellationToken">Cancellation token</param>
    /// <returns>Value or null if key not found</returns>
    public async Task<string?> GetAsync(string key, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrEmpty(key))
            throw new ArgumentException("Key cannot be null or empty", nameof(key));

        var command = $"GET {key}";
        var response = await SendCommandAsync(command, cancellationToken);

        if (response == "(null)")
            return null;

        if (response.StartsWith("ERROR "))
            throw new MerkleKvProtocolException(response.Substring(6));

        return response;
    }

    /// <summary>
    /// Deletes a key.
    /// </summary>
    /// <param name="key">Key to delete</param>
    /// <returns>True if key was deleted, false if key was not found</returns>
    public bool Delete(string key)
    {
        return DeleteAsync(key, CancellationToken.None).GetAwaiter().GetResult();
    }

    /// <summary>
    /// Deletes a key asynchronously.
    /// </summary>
    /// <param name="key">Key to delete</param>
    /// <param name="cancellationToken">Cancellation token</param>
    /// <returns>True if key was deleted, false if key was not found</returns>
    public async Task<bool> DeleteAsync(string key, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrEmpty(key))
            throw new ArgumentException("Key cannot be null or empty", nameof(key));

        var command = $"DEL {key}";
        var response = await SendCommandAsync(command, cancellationToken);

        return response switch
        {
            "DELETED" => true,
            "NOT_FOUND" => false,
            _ when response.StartsWith("ERROR ") => throw new MerkleKvProtocolException(response.Substring(6)),
            _ => throw new MerkleKvProtocolException($"Unexpected response: {response}")
        };
    }

    /// <summary>
    /// Disconnects from the server.
    /// </summary>
    private async Task DisconnectAsync()
    {
        try
        {
            _writer?.Close();
            _reader?.Close();
            _stream?.Close();
            _tcpClient?.Close();
        }
        catch
        {
            // Ignore errors during cleanup
        }
        finally
        {
            _writer = null;
            _reader = null;
            _stream = null;
            _tcpClient = null;
        }
    }

    /// <summary>
    /// Disposes the client and closes the connection.
    /// </summary>
    public void Dispose()
    {
        if (_disposed) return;

        _writer?.Close();
        _reader?.Close();
        _stream?.Close();
        _tcpClient?.Close();
        
        _disposed = true;
    }

    /// <summary>
    /// Disposes the client and closes the connection asynchronously.
    /// </summary>
    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;

        if (_writer != null) await _writer.DisposeAsync();
        if (_reader != null) _reader.Dispose();
        if (_stream != null) await _stream.DisposeAsync();
        _tcpClient?.Close();
        
        _disposed = true;
    }
}
