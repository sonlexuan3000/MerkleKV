using System;
using System.Net.Sockets;
using System.Text;

namespace MerkleKV;

/// <summary>
/// Base exception for all MerkleKV client errors.
/// </summary>
public class MerkleKvException : Exception
{
    public MerkleKvException(string message) : base(message) { }
    public MerkleKvException(string message, Exception innerException) : base(message, innerException) { }
}

/// <summary>
/// Exception thrown when a protocol error occurs (ERROR response from server).
/// </summary>
public class MerkleKvProtocolException : MerkleKvException
{
    public MerkleKvProtocolException(string message) : base(message) { }
}

/// <summary>
/// Exception thrown when a timeout occurs during network operations.
/// </summary>
public class MerkleKvTimeoutException : MerkleKvException
{
    public MerkleKvTimeoutException(string message) : base(message) { }
    public MerkleKvTimeoutException(string message, Exception innerException) : base(message, innerException) { }
}

/// <summary>
/// Exception thrown when connection-related errors occur.
/// </summary>
public class MerkleKvConnectionException : MerkleKvException
{
    public MerkleKvConnectionException(string message) : base(message) { }
    public MerkleKvConnectionException(string message, Exception innerException) : base(message, innerException) { }
}

/// <summary>
/// Optional exception for GET operations when key is not found (alternative to returning null).
/// </summary>
public class MerkleKvKeyNotFoundException : MerkleKvException
{
    public string Key { get; }
    
    public MerkleKvKeyNotFoundException(string key) : base($"Key '{key}' not found")
    {
        Key = key;
    }
}
