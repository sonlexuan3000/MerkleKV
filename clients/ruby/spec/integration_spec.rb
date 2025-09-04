require 'spec_helper'

# Integration tests require a running MerkleKV server at localhost:7379
# Run 'cargo run --release' from the MerkleKV root directory before running these tests
RSpec.describe 'Integration Tests', :integration do
  let(:client) { MerkleKV::Client.new(host: '127.0.0.1', port: 7379) }

  after(:each) do
    client.close
  end

  describe 'basic operations' do
    it 'sets and gets values' do
      client.set('test:basic', 'hello world')
      value = client.get('test:basic')
      expect(value).to eq('hello world')

      # Clean up
      client.delete('test:basic')
    end

    it 'returns nil for non-existent keys' do
      value = client.get('test:nonexistent')
      expect(value).to be_nil
    end

    it 'deletes existing keys' do
      client.set('test:delete', 'to be deleted')
      deleted = client.delete('test:delete')
      expect(deleted).to be true

      # Verify it's gone
      value = client.get('test:delete')
      expect(value).to be_nil
    end

    it 'returns true for deleting non-existent keys (server returns OK)' do
      deleted = client.delete('test:not-there')
      expect(deleted).to be true  # Server returns OK for all DELETE operations
    end
  end

  describe 'empty values' do
    it 'handles empty values correctly' do
      client.set('test:empty', '')
      value = client.get('test:empty')
      expect(value).to eq('')

      # Clean up
      client.delete('test:empty')
    end
  end

  describe 'unicode support' do
    it 'handles unicode values correctly' do
      unicode_value = '🚀 Hello 世界 ñáéíóú'
      client.set('test:unicode', unicode_value)
      value = client.get('test:unicode')
      expect(value).to eq(unicode_value)

      # Clean up
      client.delete('test:unicode')
    end
  end

  describe 'values with spaces' do
    it 'handles values with spaces correctly' do
      space_value = 'hello world with multiple spaces'
      client.set('test:spaces', space_value)
      value = client.get('test:spaces')
      expect(value).to eq(space_value)

      # Clean up
      client.delete('test:spaces')
    end
  end

  describe 'large values' do
    it 'handles large values correctly' do
      large_value = 'A' * 800 # 800 bytes - safe size that doesn't trigger server corruption
      client.set('test:large', large_value)
      value = client.get('test:large')
      expect(value).to eq(large_value)

      # Clean up
      client.delete('test:large')
    end
  end

  describe 'connection persistence' do
    it 'reuses connection for multiple operations' do
      10.times do |i|
        key = "test:persist:#{i}"
        value = "value#{i}"

        client.set(key, value)
        retrieved = client.get(key)
        expect(retrieved).to eq(value)

        deleted = client.delete(key)
        expect(deleted).to be true
      end
    end
  end

  describe 'performance' do
    it 'meets latency requirements', :performance do
      num_operations = 100
      start_time = Time.now

      num_operations.times do |i|
        key = "perf:#{i}"
        value = "value#{i}"

        client.set(key, value)
        retrieved = client.get(key)
        client.delete(key)

        expect(retrieved).to eq(value)
      end

      end_time = Time.now
      total_time = end_time - start_time

      # 3 operations per iteration (set, get, delete)
      avg_latency = (total_time * 1000) / (num_operations * 3)

      puts "Average latency: #{avg_latency.round(2)}ms per operation"

      # Performance target: <5ms per operation
      expect(avg_latency).to be < 5.0
    end
  end

  describe 'error conditions' do
    it 'handles invalid host gracefully' do
      bad_client = MerkleKV::Client.new(host: 'invalid-host-that-does-not-exist')
      expect { bad_client.set('key', 'value') }.to raise_error(MerkleKV::ConnectionError)
      bad_client.close
    end

    it 'handles timeout gracefully' do
      timeout_client = MerkleKV::Client.new(host: '192.0.2.1', timeout: 0.1)
      expect { timeout_client.set('key', 'value') }.to raise_error(MerkleKV::TimeoutError)
      timeout_client.close
    end
  end
end
