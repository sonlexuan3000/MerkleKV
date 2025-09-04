Gem::Specification.new do |spec|
  spec.name          = "merklekv"
  spec.version       = "1.0.0"
  spec.authors       = ["AI-Decenter"]
  spec.email         = ["contact@ai-decenter.com"]

  spec.summary       = "Official Ruby client for MerkleKV distributed key-value store"
  spec.description   = "Ruby client library implementing the MerkleKV TCP protocol with CRLF termination and UTF-8 support"
  spec.homepage      = "https://github.com/AI-Decenter/MerkleKV"
  spec.license       = "MIT"

  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/AI-Decenter/MerkleKV"
  spec.metadata["changelog_uri"] = "https://github.com/AI-Decenter/MerkleKV/blob/main/CHANGELOG.md"

  # Specify which files should be added to the gem when it is released.
  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{\A(?:test|spec|features)/}) }
  end
  
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Development dependencies
  spec.add_development_dependency "rspec", "~> 3.0"
  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rubocop", "~> 1.0"
end
