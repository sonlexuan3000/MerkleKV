<?php

declare(strict_types=1);

require_once __DIR__ . '/../vendor/autoload.php';

use MerkleKV\Client;

echo "MerkleKV PHP Client - Basic Usage Example\n";
echo "========================================\n\n";

try {
    // Create a client instance
    echo "1. Creating client connection to 127.0.0.1:7379...\n";
    $client = new Client("127.0.0.1", 7379, 5.0);

    // Test SET operation
    echo "2. Setting key 'user:123' to 'alice'...\n";
    $client->set("user:123", "alice");
    echo "   ✓ SET successful\n";

    // Test GET operation
    echo "3. Getting value for key 'user:123'...\n";
    $value = $client->get("user:123");
    echo "   ✓ GET result: " . ($value ?? "(null)") . "\n";

    // Test empty value
    echo "4. Setting empty value for key 'empty_key'...\n";
    $client->set("empty_key", "");
    $emptyValue = $client->get("empty_key");
    echo "   ✓ Empty value result: '" . $emptyValue . "' (length: " . strlen($emptyValue) . ")\n";

    // Test overwrite
    echo "5. Overwriting 'user:123' with 'bob'...\n";
    $client->set("user:123", "bob");
    $newValue = $client->get("user:123");
    echo "   ✓ Updated value: " . $newValue . "\n";

    // Test Unicode
    echo "6. Testing Unicode support...\n";
    $unicodeValue = "Hello, 世界! 🌍";
    $client->set("unicode_test", $unicodeValue);
    $retrievedUnicode = $client->get("unicode_test");
    echo "   ✓ Unicode value: " . $retrievedUnicode . "\n";

    // Test DELETE operation
    echo "7. Deleting key 'user:123'...\n";
    $deleted = $client->delete("user:123");
    echo "   ✓ DELETE result: " . ($deleted ? "true" : "false") . "\n";

    // Test GET after delete
    echo "8. Getting deleted key 'user:123'...\n";
    $deletedValue = $client->get("user:123");
    echo "   ✓ GET result: " . ($deletedValue ?? "(null)") . "\n";

    // Test delete non-existent key
    echo "9. Deleting non-existent key...\n";
    $notDeleted = $client->delete("non_existent_key_12345");
    echo "   ✓ DELETE result: " . ($notDeleted ? "true" : "false") . "\n";

    // Clean up test keys
    echo "10. Cleaning up test keys...\n";
    $client->delete("empty_key");
    $client->delete("unicode_test");
    echo "    ✓ Cleanup completed\n";

    // Close connection
    echo "11. Closing connection...\n";
    $client->close();
    echo "    ✓ Connection closed\n";

    echo "\n✅ All operations completed successfully!\n";

} catch (MerkleKV\ConnectionException $e) {
    echo "\n❌ Connection error: " . $e->getMessage() . "\n";
    echo "Make sure MerkleKV server is running on 127.0.0.1:7379\n";
    exit(1);
} catch (MerkleKV\TimeoutException $e) {
    echo "\n❌ Timeout error: " . $e->getMessage() . "\n";
    exit(1);
} catch (MerkleKV\ProtocolException $e) {
    echo "\n❌ Protocol error: " . $e->getMessage() . "\n";
    exit(1);
} catch (Exception $e) {
    echo "\n❌ Unexpected error: " . $e->getMessage() . "\n";
    exit(1);
}
