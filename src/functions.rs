use std::{
    path::{Path, PathBuf},
    fs::File,
    io::{Write, BufReader, BufRead},
};

use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup,
    InlineKeyboardButtonKind,
};

const COUNT_COLUMN: usize = 5;

pub fn create_folder(folder_name: &str) {
    let _ = std::fs::create_dir(folder_name);
}

pub fn create_file(file_name: &str, content: &str) {
    let mut file = File::create(file_name).unwrap();

    file.write_all(content.as_bytes()).unwrap();
}

pub fn search_files_in_directory(search_str: &str, folder_path: &str) -> Vec<PathBuf> {
    let folder_path = Path::new(folder_path);

    if !folder_path.exists() || !folder_path.is_dir() {
        return Vec::new();
    }

    let mut result = Vec::new();

    for entry in std::fs::read_dir(folder_path).unwrap() {
        let entry = entry.unwrap();

        let path = entry.path();

        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str.to_lowercase().contains(search_str) {
                    result.push(path);
                }
            }
        }
    }

    result
}

pub fn search_string_in_filenames(search_str: &str, folder_path: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let path = Path::new(folder_path);

    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(filename) = entry_path.file_name() {
                        if let Some(filename_str) = filename.to_str() {
                            if filename_str.to_lowercase().contains(search_str) {
                                result.push(entry_path.clone());
                            }
                        }
                    }
                } else if entry_path.is_dir() {
                    let mut subfolder_files = search_string_in_filenames(search_str, entry_path.to_str().unwrap());
                    result.append(&mut subfolder_files);
                }
            }
        }
    }

    result
}

pub fn search_string_inside_files(search_str: &str, folder_path: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let path = Path::new(folder_path);

    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Ok(file) = std::fs::File::open(&entry_path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines() {
                            if let Ok(line_content) = line {
                                if line_content.to_lowercase().contains(search_str) {
                                    result.push(entry_path.clone());
                                    break;
                                }
                            }
                        }
                    }
                } else if entry_path.is_dir() {
                    let mut subfolder_files = search_string_inside_files(search_str, entry_path.to_str().unwrap());
                    result.append(&mut subfolder_files);
                }
            }
        }
    }

    result
}


pub fn escape_markdown_special_chars(input: &str) -> String {
    let mut escaped_str = String::with_capacity(input.len());

    for c in input.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '<' | '>' | '#' | '+' | '-' | '.' | '!' | '|' => {
                escaped_str.push('\\');
                escaped_str.push(c);
            }
            _ => escaped_str.push(c),
        }
    }

    escaped_str
}

pub fn create_message_and_keyboard(files: Vec<PathBuf>) -> (String, InlineKeyboardMarkup) {
    let mut message = String::from("Список найденных заметок:\n");
    let mut i = 1;
    let mut count_chunks = 0;
    let mut inline_keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let mut chunk = vec![];

    for file in files {
        message.push_str(&format!("`{})` {}\n", i, escape_markdown_special_chars(&file.file_name().unwrap().to_str().unwrap().replace(".txt", ""))));

        chunk.push(InlineKeyboardButton::new(i.to_string(), InlineKeyboardButtonKind::CallbackData(file.as_os_str().to_str().unwrap().replace("Заметки\\", "").to_string())));
        i += 1;

        count_chunks += 1;
        if count_chunks == COUNT_COLUMN {
            count_chunks = 0;
            inline_keyboard.push(chunk.clone());
            chunk.clear();
        }
    }
    inline_keyboard.push(chunk.clone());

    (message, InlineKeyboardMarkup::new(inline_keyboard))
}

pub fn contains_invalid_chars(s: &str) -> bool {
    let invalid_chars = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

    for c in s.chars() {
        if invalid_chars.contains(&c) {
            return true;
        }
    }

    false
}