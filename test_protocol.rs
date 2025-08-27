fn main() {
    // Add the src directory to the path so we can import protocol
    use std::path::Path;
    use std::env;
    
    // Change to the project directory
    let project_dir = Path::new("/home/habogay/projects/MerkleKV");
    env::set_current_dir(project_dir).unwrap();
    
    // Add src to the module path and import protocol
    mod protocol;
    use protocol::Protocol;
    
    let protocol = Protocol::new();
    
    // Test the exact scenarios from the integration test
    let test_cases = vec![
        "SET key\nvalue value",
        "SET key\tvalue value", 
        "GET key\nvalue",
        "GET key\tvalue",
    ];
    
    for test_case in test_cases {
        println!("Testing: {:?}", test_case);
        match protocol.parse(test_case) {
            Ok(command) => println!("  ✓ Parsed successfully: {:?}", command),
            Err(e) => println!("  ✗ Error: {}", e),
        }
        println!();
    }
}
