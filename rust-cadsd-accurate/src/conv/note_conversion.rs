/// Convert note number to frequency in Hz (same as Python)
/// 
/// Note 69 corresponds to A4 = 440 Hz
pub fn note_to_freq(note: i32) -> f64 {
    440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
}

/// Convert frequency to note number (same as Python)
pub fn freq_to_note(freq: f64) -> i32 {
    (12.0 * (freq / 440.0).log2() + 69.0).round() as i32
}

/// Get note name from note number (same as Python)
pub fn note_name(note: i32) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let note_index = note.rem_euclid(12);
    let octave = (note - note_index) / 12 - 1;
    format!("{}{}", note_names[note_index as usize], octave)
}

/// Convert frequency to note name and cent deviation (same as Python)
pub fn freq_to_note_and_cent(freq: f64) -> (String, f64) {
    let note = freq_to_note(freq);
    let note_freq = note_to_freq(note);
    let cent = 1200.0 * (freq / note_freq).log2();
    (note_name(note), cent)
}

/// Convert frequency to wavelength in meters (same as Python)
pub fn freq_to_wavelength(freq: f64) -> f64 {
    343.0 / freq  // speed of sound at 20°C
}

/// Convert note name to note number (same as Python)
pub fn note_name_to_number(note_name: &str) -> Result<i32, String> {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    // Flat-to-sharp equivalences
    let flat_map: &[(&str, &str)] = &[
        ("Db", "C#"), ("DB", "C#"),
        ("Eb", "D#"), ("EB", "D#"),
        ("Fb", "E"),  ("FB", "E"),
        ("Gb", "F#"), ("GB", "F#"),
        ("Ab", "G#"), ("AB", "G#"),
        ("Bb", "A#"), ("BB", "A#"),
        ("Cb", "B"),  ("CB", "B"),
    ];
    
    let name = note_name.trim();
    if name.is_empty() {
        return Err("Empty note name".to_string());
    }
    
    // Extract octave digit(s) from the end
    let octave_start = name.rfind(|c: char| !c.is_ascii_digit()).map(|i| i + 1).unwrap_or(0);
    if octave_start >= name.len() {
        return Err("Invalid note format: no octave".to_string());
    }
    let octave: i32 = name[octave_start..].parse().map_err(|_| "Invalid octave".to_string())?;
    let mut note_part = name[..octave_start].to_uppercase();
    
    // Convert flats to sharps
    for (flat, sharp) in flat_map {
        if note_part == flat.to_uppercase() {
            note_part = sharp.to_string();
            break;
        }
    }
    
    // Find note index
    let note_index = note_names.iter().position(|&n| n == note_part);
    match note_index {
        Some(index) => Ok((octave + 1) * 12 + index as i32),
        None => Err(format!("Invalid note name: {}", note_part)),
    }
}

/// Calculate cent difference between two frequencies (same as Python)
pub fn cent_diff(freq1: f64, freq2: f64) -> f64 {
    1200.0 * (freq2 / freq1).log2()
}