#pragma once

#include <stdexcept>
#include <string>

namespace merklekv {

/**
 * Base exception for all MerkleKV client errors.
 */
class Exception : public std::runtime_error {
public:
    explicit Exception(const std::string& message) : std::runtime_error(message) {}
};

/**
 * Exception thrown when a connection error occurs.
 */
class ConnectionException : public Exception {
public:
    explicit ConnectionException(const std::string& message) : Exception(message) {}
};

/**
 * Exception thrown when an operation times out.
 */
class TimeoutException : public Exception {
public:
    explicit TimeoutException(const std::string& message) : Exception(message) {}
};

/**
 * Exception thrown when a protocol error occurs (ERROR response from server).
 */
class ProtocolException : public Exception {
public:
    explicit ProtocolException(const std::string& message) : Exception(message) {}
};

} // namespace merklekv
