using System;
using System.Text;
using System.Threading.Tasks;
using Xunit;

namespace MerkleKV.Tests;

public class MerkleKvClientTests
{
    [Fact]
    public void Constructor_WithDefaults_SetsCorrectValues()
    {
        using var client = new MerkleKvClient();
        // Constructor should not throw
        Assert.NotNull(client);
    }

    [Fact]
    public void Constructor_WithCustomValues_SetsCorrectValues()
    {
        using var client = new MerkleKvClient("192.168.1.1", 9999, TimeSpan.FromSeconds(10));
        Assert.NotNull(client);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public void Set_WithInvalidKey_ThrowsArgumentException(string key)
    {
        using var client = new MerkleKvClient();
        Assert.Throws<ArgumentException>(() => client.Set(key, "value"));
    }

    [Fact]
    public void Set_WithNullValue_ThrowsArgumentNullException()
    {
        using var client = new MerkleKvClient();
        Assert.Throws<ArgumentNullException>(() => client.Set("key", null!));
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public void Get_WithInvalidKey_ThrowsArgumentException(string key)
    {
        using var client = new MerkleKvClient();
        Assert.Throws<ArgumentException>(() => client.Get(key));
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    public void Delete_WithInvalidKey_ThrowsArgumentException(string key)
    {
        using var client = new MerkleKvClient();
        Assert.Throws<ArgumentException>(() => client.Delete(key));
    }

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var client = new MerkleKvClient();
        client.Dispose();
        client.Dispose(); // Should not throw
    }

    [Fact]
    public async Task DisposeAsync_CanBeCalledMultipleTimes()
    {
        var client = new MerkleKvClient();
        await client.DisposeAsync();
        await client.DisposeAsync(); // Should not throw
    }

    [Theory]
    [InlineData("")]
    [InlineData("simple")]
    [InlineData("with spaces")]
    [InlineData("with\nnewlines")]
    [InlineData("unicode: 🚀 ñ ü")]
    public void FormatValue_HandlesVariousStrings(string input)
    {
        // This tests the internal logic through the public API
        // Use a port that's definitely not in use to force connection failure
        using var client = new MerkleKvClient("127.0.0.1", 19999, TimeSpan.FromMilliseconds(100));
        // The FormatValue method is private, so we test through Set operation
        // If connection fails, we'll get connection exception, not argument exception
        var exception = Assert.Throws<MerkleKvConnectionException>(() => client.Set("test", input));
        Assert.Contains("Failed to connect", exception.Message);
    }

    [Fact]
    public void Operations_WhenDisposed_ThrowObjectDisposedException()
    {
        var client = new MerkleKvClient();
        client.Dispose();

        Assert.Throws<ObjectDisposedException>(() => client.Set("key", "value"));
        Assert.Throws<ObjectDisposedException>(() => client.Get("key"));
        Assert.Throws<ObjectDisposedException>(() => client.Delete("key"));
    }

    [Fact]
    public async Task AsyncOperations_WhenDisposed_ThrowObjectDisposedException()
    {
        var client = new MerkleKvClient();
        await client.DisposeAsync();

        await Assert.ThrowsAsync<ObjectDisposedException>(() => client.SetAsync("key", "value"));
        await Assert.ThrowsAsync<ObjectDisposedException>(() => client.GetAsync("key"));
        await Assert.ThrowsAsync<ObjectDisposedException>(() => client.DeleteAsync("key"));
    }

    [Fact]
    public void Exception_Hierarchy_IsCorrect()
    {
        var baseEx = new MerkleKvException("test");
        var protocolEx = new MerkleKvProtocolException("protocol error");
        var timeoutEx = new MerkleKvTimeoutException("timeout");
        var connectionEx = new MerkleKvConnectionException("connection error");
        var notFoundEx = new MerkleKvKeyNotFoundException("testkey");

        Assert.IsAssignableFrom<MerkleKvException>(protocolEx);
        Assert.IsAssignableFrom<MerkleKvException>(timeoutEx);
        Assert.IsAssignableFrom<MerkleKvException>(connectionEx);
        Assert.IsAssignableFrom<MerkleKvException>(notFoundEx);
        
        Assert.Equal("testkey", notFoundEx.Key);
    }
}
