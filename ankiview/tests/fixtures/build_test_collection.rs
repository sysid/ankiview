// Build script to create test collection fixture
// Run manually: cargo run --bin build_test_collection
//
// This script creates a minimal Anki collection using the Anki library.
// Due to the complexity and version-specific nature of the Anki API,
// an alternative approach is to manually create the collection in Anki desktop
// and copy it here. This script serves as documentation of what the collection should contain.

use anki::collection::CollectionBuilder;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("Creating test collection...\n");
    println!("Note: Due to Anki API complexity, this script creates an empty collection.");
    println!("You should add notes manually using Anki desktop, then copy the collection here.\n");

    let fixture_dir = PathBuf::from("tests/fixtures/test_collection");

    // Remove old collection if exists
    if fixture_dir.exists() {
        std::fs::remove_dir_all(&fixture_dir)?;
    }
    std::fs::create_dir_all(&fixture_dir)?;

    let collection_path = fixture_dir.join("collection.anki2");
    let col = CollectionBuilder::new(&collection_path).build()?;

    println!("Created empty collection at: {:?}", collection_path);

    // Close collection
    col.close(None)?;

    // Create media directory
    let media_dir = fixture_dir.join("collection.media");
    std::fs::create_dir_all(&media_dir)?;

    // Create test images
    create_test_media(&media_dir)?;

    println!("\n==============================================");
    println!("MANUAL STEPS REQUIRED:");
    println!("==============================================\n");
    println!("1. Open Anki desktop application");
    println!("2. Create a new profile or use existing one");
    println!("3. Add the following 8 notes with Basic card type:\n");
    println!("   Note 1:");
    println!("     Front: What is Rust?");
    println!("     Back: A systems programming language\n");
    println!("   Note 2:");
    println!("     Front: What is the quadratic formula?");
    println!(
        r#"     Back: <pre><code class="language-tex">$x = \frac{{-b \pm \sqrt{{b^2 - 4ac}}}}{{2a}}$</code></pre>"#
    );
    println!();
    println!("   Note 3:");
    println!("     Front: How to create a vector in Rust?");
    println!(
        r#"     Back: <pre><code class="language-rust">let v: Vec<i32> = vec![1, 2, 3];</code></pre>"#
    );
    println!();
    println!("   Note 4:");
    println!("     Front: Rust logo");
    println!(r#"     Back: <img src="rust-logo.png" alt="Rust logo">"#);
    println!();
    println!("   Note 5:");
    println!("     Front: External image test");
    println!(r#"     Back: <img src="https://example.com/test.jpg" alt="External">"#);
    println!();
    println!("   Note 6:");
    println!("     Front: HTML entities test");
    println!("     Back: Less than: &lt; Greater than: &gt; Ampersand: &amp;");
    println!();
    println!("   Note 7:");
    println!("     Front: Question with no answer");
    println!("     Back: (leave empty)");
    println!();
    println!("   Note 8:");
    println!("     Front: Tagged question");
    println!("     Back: Tagged answer");
    println!("     Tags: test rust programming");
    println!();
    println!("4. Close Anki");
    println!("5. Copy the collection.anki2 file to:");
    println!("   {}", collection_path.display());
    println!("6. Copy media files from profile's collection.media/ to:");
    println!("   {}", media_dir.display());
    println!("7. Note the IDs of the created notes (use SQLite browser or query)");
    println!("8. Update tests/helpers/mod.rs with the actual note IDs\n");
    println!("==============================================\n");

    Ok(())
}

fn create_test_media(media_dir: &std::path::Path) -> anyhow::Result<()> {
    // Create a simple 1x1 PNG file (rust-logo.png)
    // PNG signature + IHDR chunk for 1x1 red pixel
    let rust_logo_png = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, 0x02, 0x00, 0x00, 0x00, // bit depth, color type, compression, filter, interlace
        0x90, 0x77, 0x53, 0xDE, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB4, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];

    let rust_logo_path = media_dir.join("rust-logo.png");
    std::fs::write(&rust_logo_path, rust_logo_png)?;
    println!("Created test image: {:?}", rust_logo_path);

    // Create another simple PNG (sample.jpg - actually a PNG despite the name)
    let sample_path = media_dir.join("sample.jpg");
    std::fs::write(&sample_path, rust_logo_png)?;
    println!("Created test image: {:?}", sample_path);

    Ok(())
}
