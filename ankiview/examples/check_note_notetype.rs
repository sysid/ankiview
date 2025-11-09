use anki::collection::CollectionBuilder;

fn main() -> anyhow::Result<()> {
    let collection_path = "/Users/Q187392/dev/s/private/ankiview/data/testuser/collection.anki2";
    
    println!("Opening collection: {}", collection_path);
    let mut collection = CollectionBuilder::new(collection_path).build()?;
    
    // Get all notes and check their notetypes
    let all_note_ids = collection.storage.get_all_note_ids()?;
    
    println!("\nChecking first 5 Basic-type notes:");
    let mut count = 0;
    for note_id in all_note_ids {
        if count >= 5 {
            break;
        }
        
        if let Ok(Some(note)) = collection.storage.get_note(note_id) {
            if let Ok(Some(notetype)) = collection.get_notetype(note.notetype_id) {
                // Only show Basic-type notes
                if notetype.config.kind() == anki::notetype::NotetypeKind::Normal 
                   && notetype.fields.len() == 2 {
                    println!("\n{}", "=".repeat(60));
                    println!("Note ID: {}", note_id.0);
                    println!("Notetype: {} (ID: {})", notetype.name, notetype.id.0);
                    println!("Field 0 ({}): {}", notetype.fields[0].name, note.fields()[0]);
                    println!("Field 1 ({}): {}", notetype.fields[1].name, note.fields()[1]);
                    count += 1;
                }
            }
        }
    }
    
    Ok(())
}
