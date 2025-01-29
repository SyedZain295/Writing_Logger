use chrono::{Datelike, Local};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::collections::HashMap;



fn main() {

let today = Local::now().format("%Y-%m-%d").to_string();
create_daily_template("writing_log.md", &today).expect("Failed to create daily template.");



    let mut daily_stats: HashMap<String, (usize, usize)> = HashMap::new(); // Tracks word count & time (minutes)

    println!("Start writing to the log. Type 'exit' to quit.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        let word_count = input.split_whitespace().count();
        let timestamp = Local::now();
        let date_key = timestamp.format("%Y-%m-%d").to_string();

        // Update daily stats
        daily_stats
            .entry(date_key.clone())
            .and_modify(|e| {
                e.0 += word_count;
                e.1 += 1; // Assume each entry takes ~1 minute for now
            })
            .or_insert((word_count, 1));

        let log_entry = format!(
            "- **{}**: {} ({} words)\n",
            timestamp.format("%H:%M:%S"),
            input,
            word_count
        );

        save_to_file("writing_log.md", &date_key, &log_entry).expect("Failed to save log.");
        println!("Logged: {}", log_entry);
    }

    // Generate daily summaries
    generate_summary("writing_log.md", &daily_stats).expect("Failed to write summary.");
}

fn save_to_file(file_name: &str, date: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;
    file.write_all(format!("\n### {}\n", date).as_bytes())?;
    file.write_all(content.as_bytes())?;
    Ok(())
}


    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;

    file.write_all(b"\n## Summary Table\n")?;
    file.write_all(b"| Date       | Words | Time | Streak |\n")?;
    file.write_all(b"|------------|-------|------|--------|\n")?;

for (date, (words, time)) in stats {
    file.write_all(format!("| {} | {}   | {}m   | 1      |\n", date, words,time))?;
}

Ok(())
}


fn create_daily_template(file_name: &str, date: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_name)?;

    file.write_all(format!("\n## Writing Log: {}\n", date).as_bytes())?;
    file.write_all(b"- **Words Written**:\n- **Time Spent**:\n- **Topics**:\n").unwrap();
    Ok(())
}

