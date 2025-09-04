using System.Diagnostics;
using MerkleKV;

namespace Examples;

class Program
{
    static async Task Main(string[] args)
    {
        Console.WriteLine("MerkleKV .NET Client Example");
        Console.WriteLine("============================");

        // Basic usage with using statement
        Console.WriteLine("\n1. Basic Operations (async):");
        await using var client = new MerkleKvClient("127.0.0.1", 7379, TimeSpan.FromSeconds(5));
        
        try
        {
            // Set a value
            await client.SetAsync("user:1", "alice");
            Console.WriteLine("✓ Set user:1 = alice");

            // Get the value
            var value = await client.GetAsync("user:1");
            Console.WriteLine($"✓ Get user:1 = {value ?? "(null)"}");

            // Delete the key
            var deleted = await client.DeleteAsync("user:1");
            Console.WriteLine($"✓ Delete user:1 = {deleted}");

            // Try to get after delete
            var afterDelete = await client.GetAsync("user:1");
            Console.WriteLine($"✓ Get user:1 after delete = {afterDelete ?? "(null)"}");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"❌ Error: {ex.Message}");
        }

        // Synchronous operations
        Console.WriteLine("\n2. Synchronous Operations:");
        using var syncClient = new MerkleKvClient();
        
        try
        {
            syncClient.Set("sync:test", "synchronous value");
            Console.WriteLine("✓ Set sync:test synchronously");
            
            var syncValue = syncClient.Get("sync:test");
            Console.WriteLine($"✓ Get sync:test = {syncValue ?? "(null)"}");
            
            syncClient.Delete("sync:test");
            Console.WriteLine("✓ Deleted sync:test synchronously");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"❌ Sync error: {ex.Message}");
        }

        // Empty values and special characters
        Console.WriteLine("\n3. Special Values:");
        try
        {
            // Empty value (automatically handled as "")
            await client.SetAsync("empty:test", "");
            var emptyValue = await client.GetAsync("empty:test");
            Console.WriteLine($"✓ Empty value: '{emptyValue}'");

            // Unicode value
            var unicodeValue = "🚀 Hello 世界! ñáéíóú";
            await client.SetAsync("unicode:test", unicodeValue);
            var retrievedUnicode = await client.GetAsync("unicode:test");
            Console.WriteLine($"✓ Unicode value: {retrievedUnicode}");

            // Value with spaces
            var spacesValue = "value with multiple spaces";
            await client.SetAsync("spaces:test", spacesValue);
            var retrievedSpaces = await client.GetAsync("spaces:test");
            Console.WriteLine($"✓ Spaces value: '{retrievedSpaces}'");

            // Clean up
            await client.DeleteAsync("empty:test");
            await client.DeleteAsync("unicode:test");
            await client.DeleteAsync("spaces:test");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"❌ Special values error: {ex.Message}");
        }

        // Performance test
        Console.WriteLine("\n4. Performance Test (1000 operations):");
        try
        {
            var stopwatch = Stopwatch.StartNew();
            
            for (int i = 0; i < 1000; i++)
            {
                await client.SetAsync($"perf:{i}", $"value{i}");
                await client.GetAsync($"perf:{i}");
                await client.DeleteAsync($"perf:{i}");
            }
            
            stopwatch.Stop();
            var avgLatency = stopwatch.ElapsedMilliseconds / 3000.0; // 3000 total operations
            Console.WriteLine($"✓ Average latency: {avgLatency:F2}ms per operation");
            
            if (avgLatency < 5.0)
            {
                Console.WriteLine("✓ Performance target met (<5ms)");
            }
            else
            {
                Console.WriteLine("⚠️ Performance target not met (>5ms)");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"❌ Performance test error: {ex.Message}");
        }

        // Error handling demonstration
        Console.WriteLine("\n5. Error Handling:");
        
        // Connection error
        try
        {
            using var badClient = new MerkleKvClient("nonexistent-server", 7379);
            badClient.Set("test", "value");
        }
        catch (MerkleKvConnectionException ex)
        {
            Console.WriteLine($"✓ Connection error caught: {ex.Message}");
        }

        // Timeout error
        try
        {
            using var timeoutClient = new MerkleKvClient("192.0.2.1", 7379, TimeSpan.FromMilliseconds(100));
            await timeoutClient.SetAsync("test", "value");
        }
        catch (MerkleKvTimeoutException ex)
        {
            Console.WriteLine($"✓ Timeout error caught: {ex.Message}");
        }

        Console.WriteLine("\n✅ Example completed successfully!");
    }
}
