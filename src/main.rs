//
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct HistoryItem {
    pub directory_name: std::path::PathBuf,
    pub last_used:  u64,
    pub times_used: i32
}

impl Ord for HistoryItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.directory_name.cmp(&other.directory_name)
    }
}

impl PartialOrd for HistoryItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl HistoryItem {

    fn matches(&self, other: &HistoryItem) -> bool {
        let mut match_status: bool = false;
        if other.directory_name == self.directory_name {
            match_status = true;
        }
        match_status
    }

    fn is_substring_of(&self, path_component: &str, user_dir_token: &str) -> bool {
        path_component.contains(user_dir_token)
    }

    // return the order match status and the last-component-is-an-exact-match status
    //
    fn item_matches_dir_components_in_order(&self, user_dir_tokens: &Vec<String>) -> (bool, bool) {

        // 20241214-PE order_status should be true only if there is at least a partial
        // match between the end of user_dir_tokens and path_components

        let mut order_status = true; // return this
        let dir_name = self.directory_name.to_str().unwrap();
        let path_components: Vec<&str> = dir_name.split('/').collect();
        let path_components_size: i32 = path_components.len() as i32;
        let mut previously_matched_dir_component_index: i32 = -1;
        for udt in user_dir_tokens {
            let start_next_component_index = previously_matched_dir_component_index + 1;
            let mut found = false;
            if start_next_component_index < path_components_size {
                for i_pc in start_next_component_index..path_components_size {
                    let ii_pc = i_pc as usize;
                    let path_component = path_components[ii_pc];
                    if self.is_substring_of(path_component, udt) {
                        previously_matched_dir_component_index = i_pc;
                        found = true;
                        break;
                    }
                }
            }
            if ! found {
                order_status = false;
            }
        }
        let mut exact_match_last_token = false;
        if let Some(last_path_component) = path_components.last() {
            if let Some(last_user_dir_token) = user_dir_tokens.last() {
                if ! self.is_substring_of(last_path_component, last_user_dir_token) {
                    order_status = false;
                }
                if *last_path_component == last_user_dir_token.as_str() {
                    exact_match_last_token = true;
                }
            }
        }
        (order_status, exact_match_last_token)
    }

#[allow(clippy::collapsible_else_if)]
    fn get_score(&self) -> i32 {
        let debug: bool = false;
        match self.time_since_last_used() { // duration result
            Ok(duration_tslu) => {
                let tslu = duration_tslu.as_secs();
                // Use zoxide rules for weighting the score by the duration slu
                let minutes = tslu/60;
                let hours   = minutes/60;
                let days    = hours/24;
                let weeks   = days/7;
                let mut v = self.times_used * 1000;
                let tu = self.times_used;
                if weeks > 0 {
                     v /= 4;
                } else {
                    if days > 1 {
                        v /= 2;
                    } else {
                        if hours < 24 {
                            v *= 2;
                        }
                        if minutes < 60 {
                            v *= 2;
                        }
                    }
                };
                if debug {
                    println!("# {} times-used: {} minutes: {} hours: {} days: {} weeks: {}  score: {}",
                             self.directory_name.to_str().unwrap(),
                             tu, minutes, hours, days, weeks, v);
                }
                v
            },
            Err(e) => {
                println!("{}", e);
                -1
            }
        }
    }

    fn time_since_last_used(&self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        let now = std::time::SystemTime::now();
        let d: std::time::SystemTime = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.last_used);
        let diff = now.duration_since(d)?;
        Ok(diff)
    }

    fn update(&mut self) {
        self.times_used += 1;
        let now = std::time::SystemTime::now();
        match now.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(elapsed) => {
                let et: u64 = elapsed.as_secs();
                self.last_used = et;
            },
            Err(e) => {
                println!("# Error: {:?}", e);
            }
        }
    }
}

pub struct History {
    pub items: Vec<HistoryItem>
}

use std::{fs, io::prelude::*};

impl History {

    fn get_history_state_dir_name(&self) -> String {
        match std::env::var("XDG_DATA_HOME") {
            Ok(data_home_dir) => {
                let mut file_path: std::path::PathBuf = std::path::PathBuf::from(data_home_dir);
                file_path.push("z-methoxy");
                file_path.display().to_string()
            },
            Err(_e) => {
                // it's not an error if XDG_DATA_HOME is not defined - it's the normal case
                #[allow(deprecated)]
                match std::env::home_dir() {
                    None => { String::new() }, // disaster
                    Some(mut file_path) => {
                        file_path.push(".local");
                        file_path.push("share");
                        file_path.push("z-methoxy");
                        file_path.display().to_string()
                    }
                }
            }
        }
    }

    fn get_history_file_name(&self) -> String {
        let dir = self.get_history_state_dir_name();
        let mut pathbuf = std::path::PathBuf::from(dir);
        pathbuf.push("history");
        pathbuf.display().to_string()
    }

    // we write to this tmp file, them move/rename the file to the above.
    fn get_history_tmp_file_name(&self) -> String {
        let hfn = self.get_history_file_name();
        hfn + "-tmp"
    }

    fn split_line(line: String) -> HistoryItem {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() > 2 {
            let time = fields[0];
            let time_int: u64 = time.parse::<u64>().unwrap();
            let count = fields[1];
            let count_int: i32 = count.parse::<i32>().unwrap();
            let dir_name = fields[2..].join(" ");
            let pb = std::path::PathBuf::from(dir_name);
            HistoryItem { directory_name: pb, last_used: time_int, times_used: count_int }
        } else {
            let pb = std::path::PathBuf::from("");
            HistoryItem { directory_name: pb, last_used: 0, times_used: 0 }
        }
    }

    fn read_file_and_fill_history(&mut self) {
        let file_name = self.get_history_file_name();
        match std::fs::File::open(file_name.clone()) {
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                for line in reader.lines() {
                    let uline = line.unwrap();
                    let item = Self::split_line(uline);
                    self.items.push(item);
                }
            },
            Err(e) => {
                println!("# Error opening file {}: {}", file_name, e);
            }
        };
    }

    fn append_insert_item(&mut self, new_history_item: HistoryItem) {

        let mut done: bool = false;
        for item in &mut self.items {
            if item.directory_name == new_history_item.directory_name {
                item.last_used = new_history_item.last_used;
                item.times_used += 1;
                done = true;
                break;
            }
        }
        if ! done {
           self.items.push(new_history_item);
           self.items.sort();
        }
    }

    fn update_usage_of_best_item(&mut self, best_item: &HistoryItem) {
        for item in &mut self.items {
            if item.matches(best_item) { // directory is tested, of course
                item.update(); // increment usage and update used time
            }
        }
    }

    // this is the point of this program
    //
    fn find_dir_using_history(&mut self, dir_set: &Vec<String>) {

        println!("# dir_set: {:?}", dir_set);
        let dn = std::path::PathBuf::from("");
        let mut best_item = HistoryItem { directory_name: dn , last_used: 0, times_used: 0};
        let mut best_score = -1;
        for item in &self.items {
            let (order_match,last_component_match) = item.item_matches_dir_components_in_order(dir_set);
            if order_match {
                let mut score = item.get_score();
                if last_component_match { score *= 4; }
                if score > best_score {
                    best_item = item.clone();
                    best_score = score;
                }
            }
        }
        if best_score > -1 {
            println!("# best_item: {:?}", best_item);
            self.update_usage_of_best_item(&best_item);
            self.write_history();
            println!("{}", best_item.directory_name.to_str().unwrap());
        } else {
            // we found nothing, so don't go anywhere - print
            // the current dir
            match std::env::current_dir() {
                Ok(cwd) => { println!("{}", cwd.to_str().unwrap()); },
                Err(e) => {println!("# find_dir_using_history(): disasterland {}", e); }
            }
        }
    }

    // add dir to history print it
    //
    fn handle_dir(& mut self, raw_dir_set: &Vec<String>) {
        // `raw_dir` is the string that the user typed
        // println!("debug:: add_dir raw: {}", raw_dir);
        let raw_dir = raw_dir_set[0].to_string();
        self.read_file_and_fill_history();
        if std::path::Path::new(&raw_dir).is_dir() {
            // println!("debug:: found the raw dir! {}", raw_dir);
            let dir_name = std::path::PathBuf::from(raw_dir);
            let now = std::time::SystemTime::now();
            match now.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                Ok(elapsed) => {
                    let time_int: u64 = elapsed.as_secs();
                    let count_int = 1; // for a new history item
                    match std::fs::canonicalize(&dir_name) {
                        Ok(dn) => {
                            let item = HistoryItem { directory_name: dn, last_used: time_int, times_used: count_int };
                            self.append_insert_item(item);
                            self.write_history();
                            // this is what is actually used by the cd alias:
                            println!("{}", dir_name.to_str().unwrap());
                        },
                        Err(e) => {
                            println!("# Error: canonicalize {:?}", e);
                        }
                    }
                },
                Err(e) => {
                    println!("# Error: {:?}", e);
                }
            }
        } else {
            let raw_dir_string = &raw_dir;
            let raw_dir_path = std::path::Path::new(&raw_dir_string);
            if raw_dir_path.is_absolute() {
                // urgh. Failure. Was an absolute path - we don't have wiggle room
                println!("{}", &raw_dir);
            } else {
                println!("# Not found in the file system: {} - use the history to find a match", raw_dir);
                self.find_dir_using_history(raw_dir_set);
            }
        }
    }

    // and rename
    fn write_history(&self) {

        println!("#debug:: in write_history()");

        let state_dir_name = self.get_history_state_dir_name();
        let state_dir_path = std::path::PathBuf::from(&state_dir_name);
        if ! state_dir_path.exists() {
            match fs::create_dir_all(state_dir_path) {
                Ok(_s) => {},
                Err(e) => {
                    println!("# Create directory failed {} {}", &state_dir_name, e);
                }
            }
        }
        let file_name_tmp = self.get_history_tmp_file_name();
        match std::fs::File::create(&file_name_tmp) {
            Ok(mut file) => {

                let es = String::from("# Could not write to file ") + &file_name_tmp;
                for item in &self.items {
                    let last_used_str = item.last_used.to_string();
                    let times_used_str = format!("{: >4}", item.times_used);
                    file.write_all(last_used_str.as_bytes()).expect(&es);
                    file.write_all(b" ").expect(&es);
                    file.write_all(times_used_str.as_bytes()).expect(&es);
                    file.write_all(b" ").expect(&es);
                    if let Err(e) = file.write_all(item.directory_name.to_str().unwrap().to_string().as_bytes()) {
                        println!("# Failed to write PathBuf {}", e);
                    }
                    file.write_all(b"\n").expect(&es);
                }
                match file.flush() {
                    Ok(()) => {
                        // if OK, then rename
                        let file_name = self.get_history_file_name();
                        if let Err(e) = std::fs::rename(&file_name_tmp, file_name) {
                            println!("# Failed to rename {} {}", &file_name_tmp, e);
                        }
                    }
                    Err(e) => {
                        let message = "# Failed to flush";
                        println!("{} {} {}", message, &file_name_tmp, e);
                    }
                }
            },
            Err(e) => {
                let err_message = String::from("# Could not open history tmp file ") + &file_name_tmp;
                println!("{} {}", err_message, e);
            }
        }

    }

    fn handle_dash(& mut self) {

        let old_dir_result: Result<String, std::env::VarError> = std::env::var("OLD_DIR");
        let old_dir: String = old_dir_result.unwrap_or_else(|e| {
            if let std::env::VarError::NotPresent = e {
                println!("# OLD_DIR is not set");
            } else {
                eprintln!("# Failed to get OLD_DIR: {}", e);
            }
            String::new()
        });
        println!("# OLD_DIR: {}", &old_dir);
        let dir_set: Vec<String> = vec![old_dir];
        self.handle_dir(&dir_set);
    }

    fn print_items(&self, items: &Vec<HistoryItem>) {
        for item in items {
            match item.time_since_last_used() { // a duration
                Ok(duration_tslu) => {
                    let days = duration_tslu.as_secs()/(60*60*24);
                    println!("{: >4} {: >3} {}",
                             days,
                             item.times_used,
                             item.directory_name.display());
                },
                Err(e) => {
                    println!("# Failure in print_history {:?} {}", item, e);
                }
            }
        }
    }

    fn show_matches(&self, dir_set: &Vec<String>) {
        let mut mm: Vec<HistoryItem> = Vec::new();
        for item in &self.items {
            let (order_match,_last_component_match) = item.item_matches_dir_components_in_order(dir_set);
            if order_match {
                mm.push(item.clone());
            }
        }
        mm.sort();
        self.print_items(&mm);
    }

    fn print_history(&self) {
        self.print_items(&self.items);
    }
}

fn cut() {
    let mut buffer = String::new();
    loop {
        let bytes_read = std::io::stdin().read_line(&mut buffer).expect("Failed to read line");
        if bytes_read == 0 {
            break;
        }
        let item = History::split_line(buffer.clone());
        println!("{}", item.directory_name.display());
        buffer.clear();
    }
}

#[allow(clippy::collapsible_else_if)]
fn main() {

    let mut history = History { items: Vec::new(),};
    let args: Vec<_> = std::env::args().collect();
    let argc: usize = args.len();
    let debug: bool = false;
    if debug {
        println!("# number of command line arguments: {}", argc);
        // for i in 0..argc {
        for (i, arg) in args.iter().enumerate().take(argc) {
            println!("#   {} {}", i, arg);
        }
    }
    if argc > 1 {
        let first_arg: &String = &args[1];
        if debug {
            println!("# first_arg: {}", first_arg);
        }
        if first_arg == "--cut" {
            // read from standard in and remove the first 2 columns from standard out
            // This functionality is needed to use the fzf functionality in an alias
            cut();
        } else {
            if first_arg == "--print-history" {
                history.read_file_and_fill_history();
                history.print_history();
            } else {
                if first_arg == "--show-matches" {
                    history.read_file_and_fill_history();
                    let mut dir_set: Vec<String> = Vec::new();
                    for arg in args.iter().take(argc).skip(2) {
                        dir_set.push(arg.clone());
                    }
                    history.show_matches(&dir_set);
                } else {
                    if first_arg == "-" {
                        history.handle_dash();
                    } else {
                        let mut dir_set: Vec<String> = Vec::new();
                        // for i in 1..argc {
                        for arg in args.iter().take(argc).skip(1) {
                            dir_set.push(arg.to_string());
                        }
                        history.handle_dir(&dir_set);
                    }
                }
            }
        }
    } else {
        #[allow(deprecated)]
        match std::env::home_dir() {
            None => { println!("# Failed to find home directory")},
            Some(path) => {
                println!("{}", path.to_str().unwrap());
            },
        }

    }
}
