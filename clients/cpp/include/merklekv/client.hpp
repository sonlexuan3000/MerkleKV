#pragma once

#include "exceptions.hpp"
#include <chrono>
#include <optional>
#include <string>
#include <memory>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "ws2_32.lib")
using socket_t = SOCKET;
#define INVALID_SOCKET_VALUE INVALID_SOCKET
#define SOCKET_ERROR_VALUE SOCKET_ERROR
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <netdb.h>
using socket_t = int;
#define INVALID_SOCKET_VALUE -1
#define SOCKET_ERROR_VALUE -1
#define closesocket close
#endif

namespace merklekv {

/**
 * Official C++ client for MerkleKV distributed key-value store.
 * 
 * This client implements the TCP text protocol with CRLF termination and UTF-8 encoding.
 * Empty values in SET operations are automatically represented as "" at the protocol layer.
 * 
 * Example usage:
 * @code
 * MerkleKvClient client{"127.0.0.1", 7379};
 * client.set("user:1", "alice");
 * auto value = client.get("user:1");     // std::optional<std::string>
 * bool deleted = client.del("user:1");
 * @endcode
 */
class MerkleKvClient {
public:
    /**
     * Constructs a MerkleKV client.
     * 
     * @param host Server hostname or IP address (default: "127.0.0.1")
     * @param port Server port (default: 7878)
     * @param timeout Operation timeout in milliseconds (default: 5000ms)
     */
    explicit MerkleKvClient(const std::string& host = "127.0.0.1",
                           uint16_t port = 7379,
                           std::chrono::milliseconds timeout = std::chrono::milliseconds(5000));

    /**
     * Destructor - automatically closes connection.
     */
    ~MerkleKvClient();

    // Non-copyable, but movable
    MerkleKvClient(const MerkleKvClient&) = delete;
    MerkleKvClient& operator=(const MerkleKvClient&) = delete;
    MerkleKvClient(MerkleKvClient&& other) noexcept;
    MerkleKvClient& operator=(MerkleKvClient&& other) noexcept;

    /**
     * Sets a key-value pair.
     * Empty values are automatically handled - they are represented as "" at the protocol layer.
     * 
     * @param key Key to set (cannot be empty)
     * @param value Value to set (empty values are automatically converted to "")
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     * @throws ProtocolException if server returns an error
     */
    void set(const std::string& key, const std::string& value);

    /**
     * Gets a value by key.
     * 
     * @param key Key to retrieve (cannot be empty)
     * @return std::optional containing the value, or std::nullopt if key not found
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     * @throws ProtocolException if server returns an error
     */
    std::optional<std::string> get(const std::string& key);

    /**
     * Deletes a key.
     * 
     * @param key Key to delete (cannot be empty)
     * @return true if key was deleted, false if key was not found
     * @throws ConnectionException if connection fails
     * @throws TimeoutException if operation times out
     * @throws ProtocolException if server returns an error
     */
    bool del(const std::string& key);

private:
    std::string host_;
    uint16_t port_;
    std::chrono::milliseconds timeout_;
    socket_t socket_;

    void connect();
    void disconnect();
    void send_command(const std::string& command);
    std::string receive_response();
    std::string format_value(const std::string& value);
    void validate_key(const std::string& key);

#ifdef _WIN32
    class WinSockInitializer {
    public:
        WinSockInitializer();
        ~WinSockInitializer();
    };
    static WinSockInitializer winsock_init_;
#endif
};

// Implementation (header-only)
inline MerkleKvClient::MerkleKvClient(const std::string& host, uint16_t port, std::chrono::milliseconds timeout)
    : host_(host), port_(port), timeout_(timeout), socket_(INVALID_SOCKET_VALUE) {
    if (host_.empty()) {
        throw std::invalid_argument("Host cannot be empty");
    }
}

inline MerkleKvClient::~MerkleKvClient() {
    disconnect();
}

inline MerkleKvClient::MerkleKvClient(MerkleKvClient&& other) noexcept
    : host_(std::move(other.host_)), port_(other.port_), timeout_(other.timeout_), socket_(other.socket_) {
    other.socket_ = INVALID_SOCKET_VALUE;
}

inline MerkleKvClient& MerkleKvClient::operator=(MerkleKvClient&& other) noexcept {
    if (this != &other) {
        disconnect();
        host_ = std::move(other.host_);
        port_ = other.port_;
        timeout_ = other.timeout_;
        socket_ = other.socket_;
        other.socket_ = INVALID_SOCKET_VALUE;
    }
    return *this;
}

inline void MerkleKvClient::connect() {
    if (socket_ != INVALID_SOCKET_VALUE) {
        return; // Already connected
    }

    // Create socket
    socket_ = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (socket_ == INVALID_SOCKET_VALUE) {
        throw ConnectionException("Failed to create socket");
    }

    // Set timeout
    struct timeval tv;
    tv.tv_sec = timeout_.count() / 1000;
    tv.tv_usec = (timeout_.count() % 1000) * 1000;
    
#ifdef _WIN32
    DWORD timeout_ms = static_cast<DWORD>(timeout_.count());
    if (setsockopt(socket_, SOL_SOCKET, SO_RCVTIMEO, reinterpret_cast<const char*>(&timeout_ms), sizeof(timeout_ms)) == SOCKET_ERROR ||
        setsockopt(socket_, SOL_SOCKET, SO_SNDTIMEO, reinterpret_cast<const char*>(&timeout_ms), sizeof(timeout_ms)) == SOCKET_ERROR) {
        closesocket(socket_);
        socket_ = INVALID_SOCKET_VALUE;
        throw ConnectionException("Failed to set socket timeout");
    }
#else
    if (setsockopt(socket_, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) < 0 ||
        setsockopt(socket_, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof(tv)) < 0) {
        close(socket_);
        socket_ = INVALID_SOCKET_VALUE;
        throw ConnectionException("Failed to set socket timeout");
    }
#endif

    // Resolve hostname
    struct addrinfo hints = {};
    struct addrinfo* result = nullptr;
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;

    int gai_result = getaddrinfo(host_.c_str(), std::to_string(port_).c_str(), &hints, &result);
    if (gai_result != 0) {
        disconnect();
        throw ConnectionException("Failed to resolve hostname: " + host_);
    }

    // Connect
    int connect_result = ::connect(socket_, result->ai_addr, static_cast<int>(result->ai_addrlen));
    freeaddrinfo(result);

    if (connect_result == SOCKET_ERROR_VALUE) {
        disconnect();
        throw ConnectionException("Failed to connect to " + host_ + ":" + std::to_string(port_));
    }
}

inline void MerkleKvClient::disconnect() {
    if (socket_ != INVALID_SOCKET_VALUE) {
        closesocket(socket_);
        socket_ = INVALID_SOCKET_VALUE;
    }
}

inline void MerkleKvClient::send_command(const std::string& command) {
    connect();
    
    std::string full_command = command + "\r\n";
    size_t total_sent = 0;
    
    while (total_sent < full_command.size()) {
        int sent = send(socket_, full_command.c_str() + total_sent, 
                       static_cast<int>(full_command.size() - total_sent), 0);
        
        if (sent == SOCKET_ERROR_VALUE) {
            disconnect();
            throw ConnectionException("Failed to send command");
        }
        
        total_sent += sent;
    }
}

inline std::string MerkleKvClient::receive_response() {
    std::string response;
    char buffer[4096];
    
    while (true) {
        int received = recv(socket_, buffer, sizeof(buffer) - 1, 0);
        
        if (received == SOCKET_ERROR_VALUE) {
            disconnect();
            throw ConnectionException("Failed to receive response");
        }
        
        if (received == 0) {
            disconnect();
            throw ConnectionException("Server closed connection");
        }
        
        buffer[received] = '\0';
        response += buffer;
        
        // Look for CRLF terminator
        size_t crlf_pos = response.find("\r\n");
        if (crlf_pos != std::string::npos) {
            return response.substr(0, crlf_pos);
        }
    }
}

inline std::string MerkleKvClient::format_value(const std::string& value) {
    return value.empty() ? "\"\"" : value;
}

inline void MerkleKvClient::validate_key(const std::string& key) {
    if (key.empty()) {
        throw std::invalid_argument("Key cannot be empty");
    }
}

inline void MerkleKvClient::set(const std::string& key, const std::string& value) {
    validate_key(key);
    
    std::string formatted_value = format_value(value);
    std::string command = "SET " + key + " " + formatted_value;
    
    send_command(command);
    std::string response = receive_response();
    
    if (response == "OK") {
        return;
    }
    
    if (response.substr(0, 6) == "ERROR ") {
        throw ProtocolException(response.substr(6));
    }
    
    throw ProtocolException("Unexpected response: " + response);
}

inline std::optional<std::string> MerkleKvClient::get(const std::string& key) {
    validate_key(key);
    
    std::string command = "GET " + key;
    
    send_command(command);
    std::string response = receive_response();
    
    if (response == "NOT_FOUND") {
        return std::nullopt;
    }
    
    if (response.substr(0, 6) == "ERROR ") {
        throw ProtocolException(response.substr(6));
    }
    
    // Strip "VALUE " prefix from server response
    if (response.substr(0, 6) == "VALUE ") {
        std::string value = response.substr(6);
        // Handle empty value represented as ""
        if (value == "\"\"") {
            return "";
        }
        return value;
    }
    
    throw ProtocolException("Unexpected response: " + response);
}

inline bool MerkleKvClient::del(const std::string& key) {
    validate_key(key);
    
    std::string command = "DEL " + key;
    
    send_command(command);
    std::string response = receive_response();
    
    if (response == "OK") {
        return true;
    }
    
    if (response.substr(0, 6) == "ERROR ") {
        throw ProtocolException(response.substr(6));
    }
    
    throw ProtocolException("Unexpected response: " + response);
}

#ifdef _WIN32
inline MerkleKvClient::WinSockInitializer::WinSockInitializer() {
    WSADATA wsaData;
    int result = WSAStartup(MAKEWORD(2, 2), &wsaData);
    if (result != 0) {
        throw ConnectionException("WSAStartup failed");
    }
}

inline MerkleKvClient::WinSockInitializer::~WinSockInitializer() {
    WSACleanup();
}

inline MerkleKvClient::WinSockInitializer MerkleKvClient::winsock_init_{};
#endif

} // namespace merklekv
