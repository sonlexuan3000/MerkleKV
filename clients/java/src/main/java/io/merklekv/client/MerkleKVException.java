package io.merklekv.client;

/**
 * Base exception for all MerkleKV client errors.
 */
public class MerkleKVException extends Exception {
    public MerkleKVException(String message) {
        super(message);
    }

    public MerkleKVException(String message, Throwable cause) {
        super(message, cause);
    }
}

/**
 * Exception thrown when connection to server fails.
 */
class ConnectionException extends MerkleKVException {
    public ConnectionException(String message) {
        super(message);
    }

    public ConnectionException(String message, Throwable cause) {
        super(message, cause);
    }
}

/**
 * Exception thrown when operation times out.
 */
class TimeoutException extends MerkleKVException {
    public TimeoutException(String message) {
        super(message);
    }

    public TimeoutException(String message, Throwable cause) {
        super(message, cause);
    }
}

/**
 * Exception thrown when server returns an error.
 */
class ProtocolException extends MerkleKVException {
    public ProtocolException(String message) {
        super(message);
    }

    public ProtocolException(String message, Throwable cause) {
        super(message, cause);
    }
}

/**
 * Exception thrown when a key is not found.
 */
class KeyNotFoundException extends MerkleKVException {
    public KeyNotFoundException(String key) {
        super("Key not found: " + key);
    }
}
