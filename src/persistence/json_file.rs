use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use serde_json::{from_str, to_string};
use crate::domain::commands::Command;

pub fn read_commands<P: AsRef<Path>>(path: P) -> Result<Vec<Command>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    
    let mut commands = Vec::new();
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let parsed: Vec<Command> = from_str(&line)
            .map_err(|e| format!("Failed to parse command: {}", e))?;
        
        commands.extend(parsed);
    }
    
    Ok(commands)
}

pub fn write_commands<P: AsRef<Path>>(path: P, commands: &[Command]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| format!("Failed to open file for writing: {}", e))?;
    
    let json = to_string(commands).map_err(|e| format!("Failed to serialize commands: {}", e))?;
    
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write to file: {}", e))?;
    
    Ok(())
}