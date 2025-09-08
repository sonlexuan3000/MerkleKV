package merklekv

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Integration tests that require a running MerkleKV server on localhost:7379
func TestIntegrationPipeline(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err, "Server should be running on localhost:7379")

	// Test pipeline with mixed commands
	commands := []string{
		"SET pipeline_test_1 value1",
		"SET pipeline_test_2 value2", 
		"GET pipeline_test_1",
		"GET pipeline_test_2",
		"DELETE pipeline_test_1",
	}

	responses, err := client.Pipeline(commands)
	require.NoError(t, err)
	require.Len(t, responses, 5)

	// Verify responses in order
	assert.Equal(t, "OK", responses[0])      // SET
	assert.Equal(t, "OK", responses[1])      // SET
	assert.Equal(t, "VALUE value1", responses[2]) // GET
	assert.Equal(t, "VALUE value2", responses[3]) // GET
	assert.Equal(t, "DELETED", responses[4]) // DELETE
}

func TestIntegrationHealthCheck(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err, "Server should be running on localhost:7379")

	// Test health check
	healthy, err := client.HealthCheck()
	require.NoError(t, err)
	assert.True(t, healthy)
}

func TestIntegrationHealthCheckWithContext(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	healthy, err := client.HealthCheckWithContext(ctx)
	require.NoError(t, err)
	assert.True(t, healthy)
}

func TestIntegrationBasicOperations(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Test SET
	err = client.Set("integration_test", "test_value")
	require.NoError(t, err)

	// Test GET
	value, err := client.Get("integration_test")
	require.NoError(t, err)
	assert.Equal(t, "test_value", value)

	// Test DELETE
	err = client.Delete("integration_test")
	require.NoError(t, err)

	// Test GET after DELETE
	_, err = client.Get("integration_test")
	assert.Equal(t, ErrNotFound, err)
}

func TestIntegrationPing(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Test ping
	err = client.Ping()
	require.NoError(t, err)
}

func TestIntegrationEmptyValue(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Test empty value handling
	err = client.Set("empty_test", "")
	require.NoError(t, err)

	value, err := client.Get("empty_test")
	require.NoError(t, err)
	// Server returns quoted empty string for empty values
	assert.Equal(t, `""`, value)
}

func TestIntegrationConcurrentOperations(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Test that the client can handle multiple operations
	for i := 0; i < 10; i++ {
		key := "concurrent_test"
		value := "value"
		
		err = client.Set(key, value)
		require.NoError(t, err)
		
		retrieved, err := client.Get(key)
		require.NoError(t, err)
		assert.Equal(t, value, retrieved)
	}
}

func TestIntegrationPipelineWithError(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Test pipeline with an invalid command
	commands := []string{
		"SET test_key test_value",
		"INVALID_COMMAND",
	}

	_, err = client.Pipeline(commands)
	// Should get protocol error for invalid command
	assert.Error(t, err)
	
	var protocolErr *ProtocolError
	assert.True(t, errors.As(err, &protocolErr))
}

func TestIntegrationContextOperations(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	ctx := context.Background()

	// Test all context operations
	err = client.SetWithContext(ctx, "ctx_test", "value")
	require.NoError(t, err)

	value, err := client.GetWithContext(ctx, "ctx_test")
	require.NoError(t, err)
	assert.Equal(t, "value", value)

	err = client.DeleteWithContext(ctx, "ctx_test")
	require.NoError(t, err)

	err = client.PingWithContext(ctx)
	require.NoError(t, err)
}

func TestIntegrationDeleteNonExistent(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Delete non-existent key should not error
	err = client.Delete("non_existent_key_12345")
	require.NoError(t, err)
}

func TestIntegrationGetNotFound(t *testing.T) {
	client := New("localhost", 7379)
	defer client.Close()

	err := client.Connect()
	require.NoError(t, err)

	// Get non-existent key should return ErrNotFound
	_, err = client.Get("definitely_not_found_key_xyz")
	assert.Equal(t, ErrNotFound, err)
}

func TestIntegrationIsConnected(t *testing.T) {
	client := New("localhost", 7379)
	
	// Should not be connected initially
	assert.False(t, client.IsConnected())
	
	err := client.Connect()
	require.NoError(t, err)
	
	// Should be connected after Connect()
	assert.True(t, client.IsConnected())
	
	client.Close()
	
	// Should not be connected after Close()
	assert.False(t, client.IsConnected())
}
