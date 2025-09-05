require 'merklekv'

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = ".rspec_status"

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  # Only run integration tests when specifically requested
  config.filter_run_excluding :integration unless ENV['RUN_INTEGRATION_TESTS']
  config.filter_run_excluding :performance unless ENV['RUN_PERFORMANCE_TESTS']
end
