use std::fs;
use std::fs::File; // Add this import
use std::io::{self, Write, Read};
use chrono::{DateTime, Local, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use hex_literal::hex;
use std::str;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

// Struct for log entry
#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    timestamp: DateTime<Local>,
    content: String,
    word_count: usize,
    tags: Vec<String>,
    writing_time: Duration,
}

// Struct for logger
struct Logger {
    entries: Vec<Entry>,
    streaks: HashMap<String, usize>,
    daily_goal: usize,
    weekly_goal: usize,
    encryption_key: [u8; 32],
}

#[derive(Deserialize, Debug)]
struct MovieResponse {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Year")]
    year: String,
    #[serde(rename = "Rated")]
    rated: String,
    #[serde(rename = "Released")]
    released: String,
    #[serde(rename = "Runtime")]
    runtime: String,
    #[serde(rename = "Genre")]
    genre: String,
    #[serde(rename = "Director")]
    director: String,
    #[serde(rename = "Plot")]
    plot: String,
    #[serde(rename = "imdbRating")]
    imdb_rating: String,
}

impl Logger {
    // Create a new Logger
    fn new(daily_goal: usize, weekly_goal: usize, encryption_key: [u8; 32]) -> Self {
        Logger {
            entries: Vec::new(),
            streaks: HashMap::new(),
            daily_goal,
            weekly_goal,
            encryption_key,
        }
    }

    // Add a new entry
    fn add_entry(&mut self) {
        let mut content = String::new();
        println!("Enter the entry content:");
        io::stdin().read_line(&mut content).expect("Failed to read input");

        let tags: Vec<String> = vec!["tag1".to_string(), "tag2".to_string()]; // Example tags
        let word_count = content.split_whitespace().count();
        let start_time = Local::now();
        let entry = Entry {
            timestamp: start_time,
            content: content.trim().to_string(),
            word_count,
            tags,
            writing_time: Duration::zero(),
        };
        self.entries.push(entry);
        self.update_streaks();
        self.save_to_file();
        println!("Entry added successfully.");
    }

    // View entries
    fn view_entries(&self) {
        if self.entries.is_empty() {
            println!("No entries found.");
            return;
        }
        for entry in &self.entries {
            println!(
                "{} - {} ({} words) [Tags: {}]",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.content,
                entry.word_count,
                entry.tags.join(", ")
            );
        }
    }

    // Edit an entry
    fn edit_entry(&mut self) {
        if self.entries.is_empty() {
            println!("No entries to edit.");
            return;
        }
        self.view_entries();
        println!("Enter the index of the entry to edit:");
        let mut index_input = String::new();
        io::stdin().read_line(&mut index_input).expect("Failed to read input");
        let index: usize = index_input.trim().parse().expect("Invalid index");

        if index < self.entries.len() {
            let mut new_content = String::new();
            println!("Enter new content for the entry:");
            io::stdin().read_line(&mut new_content).expect("Failed to read input");
            self.entries[index].content = new_content.trim().to_string();
            self.entries[index].word_count = self.entries[index].content.split_whitespace().count();
            self.save_to_file();
            println!("Entry updated successfully.");
        } else {
            println!("Invalid index.");
        }
    }

    // Save entries to file
    fn save_to_file(&self) {
        let file_path = "log.txt";
        let mut file = fs::File::create(file_path).expect("Failed to create file");

        let serialized_entries = serde_json::to_string(&self.entries).expect("Failed to serialize entries");
        let encrypted_entries = self.encrypt(&serialized_entries);

        file.write_all(&encrypted_entries).expect("Failed to write to file");
        println!("Entries saved to {}", file_path);
    }

    // Encrypt data
    fn encrypt(&self, data: &str) -> Vec<u8> {
        let iv = hex!("000102030405060708090a0b0c0d0e0f");
        let cipher = Aes256Cbc::new_from_slices(&self.encryption_key, &iv).unwrap();
        cipher.encrypt_vec(data.as_bytes())
    }

    // Decrypt data
    fn decrypt(&self, encrypted_data: &[u8]) -> String {
        let iv = hex!("000102030405060708090a0b0c0d0e0f");
        let cipher = Aes256Cbc::new_from_slices(&self.encryption_key, &iv).unwrap();
        let decrypted_data = cipher.decrypt_vec(encrypted_data).expect("Failed to decrypt data");
        str::from_utf8(&decrypted_data).expect("Failed to convert decrypted data to string").to_string()
    }

    // Backup entries
    fn backup_entries(&self) {
        let backup_path = "backup.txt";
        let mut file = fs::File::create(backup_path).expect("Failed to create backup file");

        let serialized_entries = serde_json::to_string(&self.entries).expect("Failed to serialize entries");
        let encrypted_entries = self.encrypt(&serialized_entries);

        file.write_all(&encrypted_entries).expect("Failed to write to backup file");
        println!("Entries backed up to {}", backup_path);
    }

    // Restore entries
    fn restore_entries(&mut self) {
        let backup_path = "backup.txt";
        let mut file = fs::File::open(backup_path).expect("Failed to open backup file");

        let mut encrypted_entries = Vec::new();
        file.read_to_end(&mut encrypted_entries).expect("Failed to read backup file");

        let decrypted_entries = self.decrypt(&encrypted_entries);
        self.entries = serde_json::from_str(&decrypted_entries).expect("Failed to deserialize entries");
        println!("Entries restored from {}", backup_path);
    }

    // Search entries
    fn search_entries(&self) {
        let mut keyword = String::new();
        println!("Enter keyword to search:");
        io::stdin().read_line(&mut keyword).expect("Failed to read input");

        let keyword = keyword.trim();
        let results: Vec<&Entry> = self.entries.iter().filter(|entry| entry.content.contains(keyword)).collect();

        if results.is_empty() {
            println!("No entries found with the keyword '{}'.", keyword);
        } else {
            for entry in results {
                println!(
                    "{} - {} ({} words)",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.content,
                    entry.word_count
                );
            }
        }
    }

    // Show statistics
    fn show_statistics(&self) {
        let total_entries = self.entries.len();
        let total_words: usize = self.entries.iter().map(|entry| entry.word_count).sum();
        println!("Total entries: {}", total_entries);
        println!("Total words: {}", total_words);
    }

    // Delete an entry
    fn delete_entry(&mut self) {
        if self.entries.is_empty() {
            println!("No entries to delete.");
            return;
        }
        self.view_entries();
        println!("Enter the index of the entry to delete:");
        let mut index_input = String::new();
        io::stdin().read_line(&mut index_input).expect("Failed to read input");
        let index: usize = index_input.trim().parse().expect("Invalid index");

        if index < self.entries.len() {
            self.entries.remove(index);
            self.save_to_file();
            println!("Entry deleted successfully.");
        } else {
            println!("Invalid index.");
        }
    }

    // Sort entries by date
    fn sort_entries(&mut self) {
        self.entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        println!("Entries sorted by date.");
    }

    // Export entries to plain text
    fn export_to_plain_text(&self) {
        if self.entries.is_empty() {
            println!("No entries to export.");
            return;
        }

        let file_path = "log.txt";
        let mut file = fs::File::create(file_path).expect("Failed to create file");

        for entry in &self.entries {
            writeln!(
                file,
                "{} - {} ({} words) [Tags: {}]",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.content,
                entry.word_count,
                entry.tags.join(", ")
            ).expect("Failed to write to file");
        }
println!("Entries exported to {}", file_path);
    }

    // Search IMDb movie by title
    async fn search_movie_by_title(&self) {
        println!("Enter the movie title:");
        let mut title = String::new();
        io::stdin().read_line(&mut title).expect("Failed to read input");
        let title = title.trim();

        let url = format!("http://www.omdbapi.com/?t={}&apikey=f3523707", title);

        match reqwest::get(&url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<MovieResponse>().await {
                        Ok(movie) => {
                            println!("\nTitle: {}", movie.title);
                            println!("Year: {}", movie.year);
                            println!("Rated: {}", movie.rated);
                            println!("Released: {}", movie.released);
                            println!("Runtime: {}", movie.runtime);
                            println!("Genre: {}", movie.genre);
                            println!("Director: {}", movie.director);
                            println!("Plot: {}", movie.plot);
                            println!("IMDb Rating: {}", movie.imdb_rating);
                        }
                        Err(_) => println!("Failed to parse response."),
                    }
                } else {
                    println!("Error: {}", resp.status());
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    // Search IMDb movie by IMDb ID
    async fn search_movie_by_id(&self) {
        println!("Enter the IMDb ID:");
        let mut id = String::new();
        io::stdin().read_line(&mut id).expect("Failed to read input");
        let id = id.trim();

        let url = format!("http://www.omdbapi.com/?i={}&apikey=f3523707", id);

        match reqwest::get(&url).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<MovieResponse>().await {
                        Ok(movie) => {
                            println!("\nTitle: {}", movie.title);
                            println!("Year: {}", movie.year);
                            println!("Rated: {}", movie.rated);
                            println!("Released: {}", movie.released);
                            println!("Runtime: {}", movie.runtime);
                            println!("Genre: {}", movie.genre);
                            println!("Director: {}", movie.director);
                            println!("Plot: {}", movie.plot);
                            println!("IMDb Rating: {}", movie.imdb_rating);
                        }
                        Err(_) => println!("Failed to parse response."),
                    }
                } else {
                    println!("Error: {}", resp.status());
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    // Export entries to Markdown
    fn export_to_markdown(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;

        // Start the Markdown file
        writeln!(file, "# Writing Log\n")?;

        for entry in &self.entries {
            writeln!(file, "## {}\n", entry.timestamp.format("%Y-%m-%d %H:%M:%S"))?;
            writeln!(file, "**Word Count:** {}\n", entry.word_count)?;
            writeln!(file, "**Tags:** {}\n", entry.tags.join(", "))?;
            writeln!(file, "### Entry:\n{}\n", entry.content)?;
            writeln!(file, "---\n")?;
        }

        println!("Entries exported to {} successfully!", filename);
        Ok(())
    }

    // Update writing streaks
    fn update_streaks(&mut self) {
        let _today = Local::now().date_naive();
        let mut current_streak = 0;
        let mut longest_streak = 0;
        let mut last_date = None;

        for entry in &self.entries {
            let entry_date = entry.timestamp.date_naive();
            if let Some(last) = last_date {
                if entry_date == last + Duration::days(1) {
                    current_streak += 1;
                } else if entry_date != last {
                    current_streak = 1;
                }
            } else {
                current_streak = 1;
            }
            last_date = Some(entry_date);
            if current_streak > longest_streak {
                longest_streak = current_streak;
            }
        }

        self.streaks.insert("current".to_string(), current_streak);
        self.streaks.insert("longest".to_string(), longest_streak);
    }

    // Show writing streaks
    fn show_streaks(&self) {
        let current_streak = self.streaks.get("current").unwrap_or(&0);
        let longest_streak = self.streaks.get("longest").unwrap_or(&0);
        println!("Current streak: {} days", current_streak);
        println!("Longest streak: {} days", longest_streak);
    }

    // Show word count progress
    fn show_word_count_progress(&self) {
        let total_words_today: usize = self.entries.iter()
            .filter(|entry| entry.timestamp.date_naive() == Local::now().date_naive())
            .map(|entry| entry.word_count)
            .sum();

        let total_words_week: usize = self.entries.iter()
            .filter(|entry| entry.timestamp >= Local::now() - Duration::days(7))
            .map(|entry| entry.word_count)
            .sum();

        let daily_progress = (total_words_today as f64 / self.daily_goal as f64) * 100.0;
        let weekly_progress = (total_words_week as f64 / self.weekly_goal as f64) * 100.0;

        println!("Daily Goal: {} words", self.daily_goal);
        println!("Today's Progress: {}/{} ({:.2}%)", total_words_today, self.daily_goal, daily_progress);
        println!("Weekly Goal: {} words", self.weekly_goal);
        println!("This Week's Progress: {}/{} ({:.2}%)", total_words_week, self.weekly_goal, weekly_progress);
    }

    // Filter entries by tag
    fn filter_entries_by_tag(&self) {
        let mut tag = String::new();
        println!("Enter tag to filter by:");
        io::stdin().read_line(&mut tag).expect("Failed to read input");
        let tag = tag.trim();

        let results: Vec<&Entry> = self.entries.iter().filter(|entry| entry.tags.contains(&tag.to_string())).collect();

        if results.is_empty() {
            println!("No entries found with the tag '{}'.", tag);
        } else {
            for entry in results {
                println!(
                    "{} - {} ({} words) [Tags: {}]",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.content,
                    entry.word_count,
                    entry.tags.join(", ")
                );
            }
        }
    }

    // Filter entries by date range
    fn filter_entries_by_date_range(&self) {
        let mut start_date = String::new();
        let mut end_date = String::new();
        println!("Enter start date (YYYY-MM-DD):");
        io::stdin().read_line(&mut start_date).expect("Failed to read input");
        println!("Enter end date (YYYY-MM-DD):");
        io::stdin().read_line(&mut end_date).expect("Failed to read input");

        let start_date = DateTime::parse_from_str(&start_date.trim(), "%Y-%m-%d").expect("Invalid date format").date_naive();
        let end_date = DateTime::parse_from_str(&end_date.trim(), "%Y-%m-%d").expect("Invalid date format").date_naive();

        let results: Vec<&Entry> = self.entries.iter()
            .filter(|entry| entry.timestamp.date_naive() >= start_date && entry.timestamp.date_naive() <= end_date)
            .collect();

        if results.is_empty() {
            println!("No entries found in the date range '{} - {}'.", start_date, end_date);
        } else {
            for entry in results {
                println!(
                    "{} - {} ({} words) [Tags: {}]",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.content,
                    entry.word_count,
                    entry.tags.join(", ")
                );
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let encryption_key = hex!("000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f");
    let mut logger = Logger::new(500, 3500, encryption_key); // Example daily and weekly goals

    loop {
        println!("--- Writing Logger ---");
        println!("1. Add a new entry");
        println!("2. View entries");
        println!("3. Edit an entry");
        println!("4. Search entries");
        println!("5. Save entries to file");
        println!("6. View statistics");
        println!("7. Delete an entry");
        println!("8. Sort entries by date");
        println!("9. Export entries to plain text");
        println!("10. Search IMDb Movie (by Title)");
        println!("11. Search IMDb Movie (by IMDb ID)");
        println!("12. Export entries to Markdown");
        println!("13. Show writing streaks");
        println!("14. Show word count progress");
        println!("15. Filter entries by tag");
        println!("16. Filter entries by date range");
        println!("17. Backup entries");
        println!("18. Restore entries");
        println!("19. Exit");
        println!("Enter Your Choice: ");

        let mut choice = String::new();
io::stdin().read_line(&mut choice).expect("Failed to read input");

        match choice.trim() {
            "1" => logger.add_entry(),
            "2" => logger.view_entries(),
            "3" => logger.edit_entry(),
            "4" => logger.search_entries(),
            "5" => logger.save_to_file(),
            "6" => logger.show_statistics(),
            "7" => logger.delete_entry(),
            "8" => logger.sort_entries(),
            "9" => logger.export_to_plain_text(),
            "10" => logger.search_movie_by_title().await,
            "11" => logger.search_movie_by_id().await,
            "12" => {
                if let Err(e) = logger.export_to_markdown("writing_log.md") {
                    println!("Failed to export: {}", e);
                }
            }
            "13" => logger.show_streaks(),
            "14" => logger.show_word_count_progress(),
            "15" => logger.filter_entries_by_tag(),
            "16" => logger.filter_entries_by_date_range(),
            "17" => logger.backup_entries(),
            "18" => logger.restore_entries(),
            "19" => {
                println!("Exiting the logger. Goodbye!");
                break;
            }
            _ => {
                println!("Invalid choice. Please try again.");
            }
        }
    }
}

