using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Xunit;

namespace MerkleKV.Tests;

/// <summary>
/// Integration tests that require a running MerkleKV server at localhost:7379
/// Run 'cargo run --release' from the MerkleKV root directory before running these tests.
/// </summary>
public class IntegrationTests
{
    private const string TestHost = "127.0.0.1";
    private const int TestPort = 7379;

    [Fact]
    public async Task BasicOperations_WorkCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        // Set and get
        await client.SetAsync("test:basic", "hello world");
        var value = await client.GetAsync("test:basic");
        Assert.Equal("hello world", value);

        // Delete
        var deleted = await client.DeleteAsync("test:basic");
        Assert.True(deleted);

        // Get after delete
        var valueAfterDelete = await client.GetAsync("test:basic");
        Assert.Null(valueAfterDelete);

        // Delete non-existent
        var deletedNonExistent = await client.DeleteAsync("test:nonexistent");
        Assert.False(deletedNonExistent);
    }

    [Fact]
    public async Task EmptyValue_HandledCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        // Set empty value
        await client.SetAsync("test:empty", "");
        var value = await client.GetAsync("test:empty");
        Assert.Equal("", value);

        // Clean up
        await client.DeleteAsync("test:empty");
    }

    [Fact]
    public async Task UnicodeValues_HandledCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        var unicodeValue = "🚀 Hello 世界 ñáéíóú";
        await client.SetAsync("test:unicode", unicodeValue);
        var value = await client.GetAsync("test:unicode");
        Assert.Equal(unicodeValue, value);

        // Clean up
        await client.DeleteAsync("test:unicode");
    }

    [Fact]
    public async Task ValuesWithSpaces_HandledCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        var spaceValue = "hello world with spaces";
        await client.SetAsync("test:spaces", spaceValue);
        var value = await client.GetAsync("test:spaces");
        Assert.Equal(spaceValue, value);

        // Clean up
        await client.DeleteAsync("test:spaces");
    }

    [Fact]
    public async Task LargeValue_HandledCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        var largeValue = new string('A', 10000); // 10KB
        await client.SetAsync("test:large", largeValue);
        var value = await client.GetAsync("test:large");
        Assert.Equal(largeValue, value);

        // Clean up
        await client.DeleteAsync("test:large");
    }

    [Fact]
    public void SyncOperations_WorkCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        // Test sync operations
        client.Set("test:sync", "sync value");
        var value = client.Get("test:sync");
        Assert.Equal("sync value", value);

        var deleted = client.Delete("test:sync");
        Assert.True(deleted);
    }

    [Fact]
    public async Task ConnectionTimeout_HandledCorrectly()
    {
        // Test with invalid host to trigger timeout
        using var client = new MerkleKvClient("192.0.2.1", 7379, TimeSpan.FromMilliseconds(100));
        
        await Assert.ThrowsAsync<MerkleKvTimeoutException>(() => 
            client.SetAsync("test", "value"));
    }

    [Fact]
    public async Task InvalidHost_ThrowsConnectionException()
    {
        // Test with invalid host
        using var client = new MerkleKvClient("invalid-host-that-does-not-exist", 7379);
        
        await Assert.ThrowsAsync<MerkleKvConnectionException>(() => 
            client.SetAsync("test", "value"));
    }

    [Fact]
    public async Task ConcurrentOperations_WorkCorrectly()
    {
        using var client = new MerkleKvClient(TestHost, TestPort);
        
        var tasks = new List<Task>();
        
        // Run multiple operations concurrently
        for (int i = 0; i < 10; i++)
        {
            var index = i;
            tasks.Add(Task.Run(async () =>
            {
                await client.SetAsync($"test:concurrent:{index}", $"value{index}");
                var value = await client.GetAsync($"test:concurrent:{index}");
                Assert.Equal($"value{index}", value);
                await client.DeleteAsync($"test:concurrent:{index}");
            }));
        }
        
        await Task.WhenAll(tasks);
    }

    [Fact]
    public async Task UsingStatement_DisposesCorrectly()
    {
        await using var client = new MerkleKvClient(TestHost, TestPort);
        await client.SetAsync("test:using", "test");
        var value = await client.GetAsync("test:using");
        Assert.Equal("test", value);
        await client.DeleteAsync("test:using");
        
        // Client should be disposed automatically
    }
}
