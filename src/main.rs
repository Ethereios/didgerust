use cadsd::Geo;

fn main() {
    println!("CADSD - Didgeridoo Analysis");
    println!("Running basic analysis...");
    
    // Example usage
    let geo = Geo::make_cone(1500.0, 32.0, 65.0, 30);
    println!("Created cone: {}mm long, {}mm bell", geo.length(), geo.bellsize());
}