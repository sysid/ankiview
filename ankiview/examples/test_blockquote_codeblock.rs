use ankiview::inka::infrastructure::markdown::card_parser;
use ankiview::inka::infrastructure::markdown::converter;

fn main() {
    // Simulate the exact markdown structure from persistence.md
    let note_str = r#"<!--ID:1689963334894-->
1. What is Isolation Level in DB?
> Isolation controls:
> - Whether locks are taken when data is read
>
> **Dirty reads** - A dirty read occurs when a transaction retrieves a row.
> ```sql
> BEGIN;
> SELECT age FROM users WHERE id = 1;
> -- retrieves 20
> COMMIT;
> ```"#;

    println!("=== ORIGINAL NOTE STRING ===");
    println!("{}", note_str);
    println!();

    // Step 1: Parse the card to extract front and back
    let (front_md, back_md) = card_parser::parse_basic_card_fields(note_str).unwrap();
    
    println!("=== FRONT MARKDOWN ===");
    println!("{}", front_md);
    println!();
    
    println!("=== BACK MARKDOWN (after clean_answer) ===");
    println!("{}", back_md);
    println!();

    // Step 2: Convert to HTML
    let back_html = converter::markdown_to_html(&back_md);
    
    println!("=== BACK HTML ===");
    println!("{}", back_html);
    println!();
    
    // Check if code block is preserved
    if back_html.contains("<pre") && back_html.contains("<code") {
        println!("✓ Code block preserved as <pre><code>");
    } else {
        println!("✗ Code block NOT preserved!");
    }
}
