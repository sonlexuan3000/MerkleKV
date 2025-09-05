require 'spec_helper'

RSpec.describe MerkleKV::Client do
  describe '#initialize' do
    it 'creates client with default values' do
      client = MerkleKV::Client.new
      expect(client).not_to be_nil
    end

    it 'creates client with custom values' do
      client = MerkleKV::Client.new(host: '192.168.1.1', port: 9999, timeout: 10.0)
      expect(client).not_to be_nil
    end
  end

  describe '#set' do
    let(:client) { MerkleKV::Client.new }

    it 'raises ArgumentError for nil key' do
      expect { client.set(nil, 'value') }.to raise_error(ArgumentError, 'Key cannot be nil')
    end

    it 'raises ArgumentError for empty key' do
      expect { client.set('', 'value') }.to raise_error(ArgumentError, 'Key cannot be empty')
    end

    it 'raises ArgumentError for nil value' do
      expect { client.set('key', nil) }.to raise_error(ArgumentError, 'Value cannot be nil')
    end
  end

  describe '#get' do
    let(:client) { MerkleKV::Client.new }

    it 'raises ArgumentError for nil key' do
      expect { client.get(nil) }.to raise_error(ArgumentError, 'Key cannot be nil')
    end

    it 'raises ArgumentError for empty key' do
      expect { client.get('') }.to raise_error(ArgumentError, 'Key cannot be empty')
    end
  end

  describe '#delete' do
    let(:client) { MerkleKV::Client.new }

    it 'raises ArgumentError for nil key' do
      expect { client.delete(nil) }.to raise_error(ArgumentError, 'Key cannot be nil')
    end

    it 'raises ArgumentError for empty key' do
      expect { client.delete('') }.to raise_error(ArgumentError, 'Key cannot be empty')
    end
  end

  describe '#close' do
    let(:client) { MerkleKV::Client.new }

    it 'can be called multiple times safely' do
      expect { client.close }.not_to raise_error
      expect { client.close }.not_to raise_error
    end
  end

  describe '#connected?' do
    let(:client) { MerkleKV::Client.new }

    it 'returns false when not connected' do
      expect(client.connected?).to be false
    end
  end

  describe 'connection errors' do
    it 'raises ConnectionError for invalid host' do
      client = MerkleKV::Client.new(host: 'invalid-host-that-does-not-exist')
      expect { client.set('key', 'value') }.to raise_error(MerkleKV::ConnectionError)
    end

    it 'raises TimeoutError for connection timeout' do
      client = MerkleKV::Client.new(host: '192.0.2.1', timeout: 0.1)
      expect { client.set('key', 'value') }.to raise_error(MerkleKV::TimeoutError)
    end
  end

  describe 'exception hierarchy' do
    it 'has correct inheritance' do
      expect(MerkleKV::ConnectionError.superclass).to eq(MerkleKV::Error)
      expect(MerkleKV::TimeoutError.superclass).to eq(MerkleKV::Error)
      expect(MerkleKV::ProtocolError.superclass).to eq(MerkleKV::Error)
      expect(MerkleKV::Error.superclass).to eq(StandardError)
    end
  end
end
