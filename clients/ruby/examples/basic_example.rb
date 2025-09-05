#!/usr/bin/env ruby

require 'merklekv'

puts "MerkleKV Ruby Client Example"
puts "============================"
puts

begin
  # Create client with custom timeout
  client = MerkleKV::Client.new(host: "127.0.0.1", port: 7379, timeout: 5.0)

  puts "1. Basic Operations:"
  
  # Set a value
  client.set("user:1", "alice")
  puts "✓ Set user:1 = alice"

  # Get the value
  value = client.get("user:1")
  puts "✓ Get user:1 = #{value || '(nil)'}"

  # Delete the key
  deleted = client.delete("user:1")
  puts "✓ Delete user:1 = #{deleted}"

  # Try to get after delete
  after_delete = client.get("user:1")
  puts "✓ Get user:1 after delete = #{after_delete || '(nil)'}"

  puts
  puts "2. Special Values:"
  
  # Empty value (automatically handled as "")
  client.set("empty:test", "")
  empty_value = client.get("empty:test")
  puts "✓ Empty value: '#{empty_value}'"

  # Unicode value
  unicode_value = "🚀 Hello 世界! ñáéíóú"
  client.set("unicode:test", unicode_value)
  retrieved_unicode = client.get("unicode:test")
  puts "✓ Unicode value: #{retrieved_unicode}"

  # Value with spaces
  spaces_value = "value with multiple spaces"
  client.set("spaces:test", spaces_value)
  retrieved_spaces = client.get("spaces:test")
  puts "✓ Spaces value: '#{retrieved_spaces}'"

  # Clean up
  client.delete("empty:test")
  client.delete("unicode:test")
  client.delete("spaces:test")

  puts
  puts "3. Performance Test (1000 operations):"
  
  start_time = Time.now
  
  1000.times do |i|
    key = "perf:#{i}"
    value = "value#{i}"
    
    client.set(key, value)
    client.get(key)
    client.delete(key)
  end
  
  end_time = Time.now
  total_time = end_time - start_time
  
  avg_latency = (total_time * 1000) / 3000.0 # 3000 total operations
  puts "✓ Average latency: #{'%.2f' % avg_latency}ms per operation"
  
  if avg_latency < 5.0
    puts "✓ Performance target met (<5ms)"
  else
    puts "⚠️ Performance target not met (>5ms)"
  end

  puts
  puts "4. Connection Management:"
  
  # Show connection status
  puts "✓ Connected: #{client.connected?}"
  
  # Multiple operations on same connection
  5.times do |i|
    client.set("multi:#{i}", "value#{i}")
    value = client.get("multi:#{i}")
    client.delete("multi:#{i}")
    puts "✓ Operation #{i + 1} completed: #{value}"
  end

  # Close connection
  client.close
  puts "✓ Connection closed: #{client.connected?}"

rescue MerkleKV::ConnectionError => e
  puts "❌ Connection error: #{e.message}"
  puts "   Make sure MerkleKV server is running: cargo run --release"
  exit 1
rescue MerkleKV::TimeoutError => e
  puts "❌ Timeout error: #{e.message}"
  exit 1
rescue MerkleKV::ProtocolError => e
  puts "❌ Protocol error: #{e.message}"
  exit 1
rescue MerkleKV::Error => e
  puts "❌ MerkleKV error: #{e.message}"
  exit 1
rescue StandardError => e
  puts "❌ Unexpected error: #{e.message}"
  exit 1
ensure
  # Ensure client is closed
  client&.close
end

puts
puts "✅ Example completed successfully!"
