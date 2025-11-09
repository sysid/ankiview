use anki::collection::CollectionBuilder;

fn main() -> anyhow::Result<()> {
    let collection_path = "/Users/Q187392/dev/s/private/ankiview/data/testuser/collection.anki2";
    
    println!("Opening collection: {}", collection_path);
    let mut collection = CollectionBuilder::new(collection_path).build()?;
    
    let all_notetypes = collection.get_all_notetypes()?;
    
    println!("\nFound {} notetypes:", all_notetypes.len());
    for notetype in all_notetypes {
        println!("\n{}", "=".repeat(60));
        println!("Notetype: {}", notetype.name);
        println!("{}", "=".repeat(60));
        println!("  ID: {}", notetype.id.0);
        println!("  Kind: {:?}", notetype.config.kind());
        println!("\n  Fields ({}):", notetype.fields.len());
        for (i, field) in notetype.fields.iter().enumerate() {
            println!("    [{}] {}", i, field.name);
        }
        println!("\n  Templates ({}):", notetype.templates.len());
        for (i, template) in notetype.templates.iter().enumerate() {
            println!("    [{}] {}", i, template.name);
        }
    }
    
    Ok(())
}
