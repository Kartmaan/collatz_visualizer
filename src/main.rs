mod collatz;

use iced::{
    widget::{
        button, column, container, row, text, text_input, vertical_space, horizontal_space,
        scrollable, image,
    },
    executor, Application, Command, Element, Length, Settings, Theme, Color, Alignment,
};
use plotters::prelude::*; // Drawing charts.
use plotters::style::Color as PlottersColor; // To avoid conflicts with iced::Color.
use rand::Rng; // Random number generation
use std::path::PathBuf; // Working with file paths.
use clipboard::{ClipboardContext, ClipboardProvider}; // Copying text to the system clipboard.
use chrono::Local; // Getting the current date and time (used for filenames).
use std::fs; // Standard library file system utilities.

// ==========================================================================
//                              Application State
// ==========================================================================
// Main structure that holds the application's state.
pub struct CollatzApp {
    // Input fields
    // String to hold the text entered in the 1st and 2nd input box.
    input1: String,
    input2: String,
    
    // Processed values
    // Option<u64> holds the parsed integer value from input1/input2, if valid. None otherwise.
    value1: Option<u64>,
    value2: Option<u64>,
    
    // Calculated Sequences
    // Vectors to store the generated Collatz sequence for value1/value2.
    sequence1: Vec<u64>,
    sequence2: Vec<u64>,
    
    // Statistics
    // Option containing statistics for sequence1/sequence2, if calculated.
    stats1: Option<collatz::CollatzStats>,
    stats2: Option<collatz::CollatzStats>,
    
    // Application State Flags
    error_message: String, // String to display error messages to the user.
    chart_saved: bool, // Flag to indicate if the chart was successfully saved recently.
    copied_to_clipboard: bool, // Flag to indicate if the sequences were successfully copied recently.
    
    // Chart
    // Option storing the file path to the currently generated chart image.
    // This is likely a temporary file until saved permanently.
    chart_path: Option<String>,
}

// ==========================================================================
//                               Messages (events)
// ==========================================================================
// Define the messages that can be sent to the application's update function.
// These represent events or user actions.
#[derive(Debug, Clone)]
pub enum Message {
    Input1Changed(String), // Text in the 1st input box changes. Contains the new text.
    Input2Changed(String), // Text in the 2nd input box changes. Contains the new text.
    Visualize, // "Visualize" button is pressed.
    Randomize, // "Randomize" button is pressed.
    SaveChart, // "Save Chart" button is pressed.
    CopyToClipboard, // "Copy" button is pressed.
    
    // Message sent *after* the chart generation task completes.
    // Contains Ok(path_string) on success, or Err(error_message) on failure.
    ChartGenerated(Result<String, String>),

    // Message sent *after* the chart saving task completes.
    // Contains Ok(()) on success, or Err(error_message) on failure.
    ChartSaved(Result<(), String>),

    // Message sent *after* the clipboard copy task completes.
    // Contains Ok(()) on success, or Err(error_message) on failure.
    ClipboardCopied(Result<(), String>),

    // Message sent *after* the old temporary file cleanup task completes.
    // Contains Ok(()) on success, or Err(error_message) on failure.
    CleanupOldTempFiles(Result<(), String>),
}

// ==========================================================================
//                              Application Setup
// ==========================================================================
// Implement the Iced Application trait for our CollatzApp struct.
impl Application for CollatzApp {
    // Specifies the type of executor to use for running commands (async tasks).
    // `executor::Default` is suitable for most desktop applications.
    type Executor = executor::Default; // The type of messages our application understands.
    type Message = Message; // The type of messages our application understands. 
    type Theme = Theme; // The theme used for styling the application. Using the default Iced theme.
    type Flags = (); // Flags are data that can be passed to the application on startup (we don't use any).

    /// Called once when the application starts.
    /// Initializes the application state (`Self`) and can return an initial `Command`.
    /// The command can be used to perform async tasks or send messages.
    /// In this case, we don't need to perform any async tasks at startup, so we return `Command::none()`.
    /// The `flags` parameter can be used to pass data to the application on startup.
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Return the initial state of the application.
        (
            Self {
                // Initialize input strings as empty.
                input1: String::new(),
                input2: String::new(),

                // Initialize optional values as None (no values yet).
                value1: None,
                value2: None,

                // Initialize sequences as empty vectors.
                sequence1: Vec::new(),
                sequence2: Vec::new(),

                // Initialize statistics as None.
                stats1: None,
                stats2: None,

                error_message: String::new(), // Initialize error message as empty.
                chart_saved: false, // Initialize flags as false.
                copied_to_clipboard: false, // Nothing copied on clipboard yet
                chart_path: None, // Not chart yet
            },
            // No initial command needs to be run when the application starts.
            Command::none(),
        )
    }

    /// Determines the title of the application window.
    /// This function is called whenever the state changes, allowing for dynamic titles.
    /// The title is constructed based on the current state of the application.
    /// It includes the Collatz conjecture visualizer title and the values entered by the user.
    /// If no values are entered, the title will just be "Collatz Conjecture Visualizer".
    /// If one or both values are entered, they will be appended to the title.
    fn title(&self) -> String {
        // Start with a base title.
        let mut title = String::from("Collatz Conjecture Visualizer");
        
        // Append the first value if it exists.
        if let Some(v1) = self.value1 {
            title.push_str(&format!(" - {}", v1));
            
            // Append the second value if it also exists.
            if let Some(v2) = self.value2 {
                title.push_str(&format!(" and {}", v2));
            }
        }
        
        title // Return the constructed title string.
    }

    // ==========================================================================
    //                              Update Function
    // ==========================================================================
    /// Handles messages sent to the application (e.g., from user interactions).
    /// This function updates the application's state (`self`) based on the message
    /// and can return a `Command` to perform further actions (like async tasks).
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // --- Input Handling ---
            // When the text in the first input box changes, update the input1 field in the state.
            Message::Input1Changed(value) => {
                // Update the input1 field in the state with the new text.
                self.input1 = value;
                // No further command needed.
                Command::none()
            }
            
            // When the text in the second input box changes, update the input2 field in the state.
            Message::Input2Changed(value) => {
                // Update the input2 field in the state with the new text.
                self.input2 = value;
                // No further command needed.
                Command::none()
            }
            
            // --- Core Actions ---
            // When the "Visualize" button is pressed, we need to process the inputs.
            // This includes parsing the inputs, generating the Collatz sequences,
            // and creating the chart.
            Message::Visualize => {
                // Reset status messages and flags before processing.
                self.error_message = String::new();
                self.chart_saved = false;
                self.copied_to_clipboard = false;
                
                // Processing the first input
                // Parse the first input as a u64 integer.
                // If parsing fails, set the error message.
                // If parsing succeeds, generate the Collatz sequence and calculate statistics.
                match self.input1.trim().parse::<u64>() {
                    Ok(value) => {
                        if value == 0 { // Check if the value is greater than 0
                            self.error_message = "The first value must be greater than 0".to_string();
                            return Command::none();
                        }
                        
                        self.value1 = Some(value); // Parse the input as a u64.
                        self.sequence1 = collatz::generate_sequence(value); // Generate the Collatz sequence.
                        self.stats1 = Some(collatz::calculate_stats(&self.sequence1)); // Calculate statistics.
                    }

                    // If parsing fails, check if the input is empty.
                    Err(_) => {
                        if !self.input1.trim().is_empty() {
                            self.error_message = "Invalid first value".to_string();
                        } else {
                            self.value1 = None;
                            self.sequence1.clear();
                            self.stats1 = None;
                        }
                    }
                }
                
                // Processing the second input
                match self.input2.trim().parse::<u64>() {
                    Ok(value) => {
                        if value == 0 { // Check if the value is greater than 0
                            self.error_message = "The value must be greater than 0".to_string();
                            return Command::none();
                        }
                        
                        self.value2 = Some(value);
                        self.sequence2 = collatz::generate_sequence(value);
                        self.stats2 = Some(collatz::calculate_stats(&self.sequence2));
                    }
                    // If parsing fails, check if the input is empty.
                    Err(_) => {
                        if !self.input2.trim().is_empty() {
                            self.error_message = "Invalid second value".to_string();
                        } else {
                            self.value2 = None;
                            self.sequence2.clear();
                            self.stats2 = None;
                        }
                    }
                }
                
                // If at least one sequence is generated, proceed to generate the chart.
                // If both sequences are empty, do nothing.
                if !self.sequence1.is_empty() || !self.sequence2.is_empty() {
                    // Delete the old temporary file if it exists.
                    // This is done to avoid cluttering the directory with old files.
                    // If the chart_path is None, it means no chart was generated yet.
                    let cleanup_command = if let Some(old_path) = &self.chart_path {
                        Command::perform(
                            cleanup_temp_file(old_path.clone()),
                            Message::CleanupOldTempFiles,
                        )
                    } else {
                        Command::none()
                    };
                    
                    // Generate a new filename for the chart.
                    // Use the current date and time to ensure uniqueness.
                    let now = Local::now();
                    let filename = format!("temp_collatz_{}.png", now.format("%Y%m%d_%H%M%S"));
                    
                    // Generate the chart and save it to the temporary file.
                    // The chart generation is an async task, so we use Command::perform.
                    // The result of the task will be sent back as a Message::ChartGenerated.
                    // The chart will be generated with the sequences and values provided.
                    let generate_command = Command::perform(
                        generate_chart(
                            PathBuf::from(&filename),
                            self.value1,
                            self.value2,
                            self.sequence1.clone(),
                            self.sequence2.clone(),
                        ),
                        Message::ChartGenerated,
                    );
                    
                    // Return a batch command that performs both cleanup and chart generation.
                    // This allows both tasks to run concurrently.
                    // The cleanup command will run first, and then the chart generation.
                    // This is a good practice to ensure we don't leave old temporary files behind.
                    Command::batch(vec![cleanup_command, generate_command])
                } else {
                    Command::none() // No command needed if no sequences are generated.
                }
            }
            
            // When the "Randomize" button is pressed, generate two random numbers
            // between 1 and 10000 (inclusive) and set them as the input values.
            // Then, call the Visualize function to generate the sequences and chart.
            Message::Randomize => {
                let mut rng = rand::thread_rng(); // Create a random number generator.
                
                let max_rand = 10000; // Maximum random number.
                let random1 = rng.gen_range(1..=max_rand);
                let random2 = rng.gen_range(1..=max_rand);
                
                // Set the random values as input strings.
                // This will update the input fields in the UI.
                self.input1 = random1.to_string();
                self.input2 = random2.to_string();
                
                // Call the Visualize function to generate the sequences and chart.
                // This is done by sending a Message::Visualize.
                // The Visualize function will parse the inputs and generate the sequences.
                // If the inputs are valid, it will also generate the chart.
                self.update(Message::Visualize)
            }
            
            // When the "Save Chart" button is pressed, we need to save the generated chart.
            // If no chart was generated, show an error message.
            // If a chart was generated, copy it to a new file with a timestamped name.
            Message::SaveChart => {
                // Check if there are sequences to save.
                // If both sequences are empty, show an error message.
                if self.sequence1.is_empty() && self.sequence2.is_empty() {
                    self.error_message = "Ne sequence to save".to_string();
                    return Command::none();
                }
                
                // Check if a chart was generated.
                // If no chart was generated, show an error message.
                // The chart_path is an Option<String>, so we need to check if it's Some.
                // If it's None, it means no chart was generated yet.
                if self.chart_path.is_none() {
                    self.error_message = "No graph to save".to_string();
                    return Command::none();
                }
                
                self.chart_saved = false; // Reset the chart saved flag before saving.
                
                // Generate a new filename for the saved chart.
                // Use the current date and time to ensure uniqueness.
                let now = Local::now();
                let filename = format!("collatz_{}.png", now.format("%Y%m%d_%H%M%S"));
                
                // Create a command to save the chart.
                // This is an async task, so we use Command::perform.
                // The result of the task will be sent back as a Message::ChartSaved.
                // The save_chart function will copy the temporary chart file to a new file.
                Command::perform(
                    save_chart(
                        self.chart_path.clone().unwrap(),
                        filename,
                    ),
                    Message::ChartSaved,
                )
            }
            
            // When the "Copy to Clipboard" button is pressed, we need to copy the sequences
            // to the system clipboard.
            // If no sequences were generated, show an error message.
            // If sequences were generated, format them and copy them to the clipboard.
            Message::CopyToClipboard => {
                if self.sequence1.is_empty() && self.sequence2.is_empty() {
                    self.error_message = "No sequence to copy".to_string();
                    return Command::none();
                }
                
                self.copied_to_clipboard = false; // Reset the copied to clipboard flag before copying.
                
                // Create a command to copy the sequences to the clipboard.
                // This is an async task, so we use Command::perform.
                // The result of the task will be sent back as a Message::ClipboardCopied.
                Command::perform(
                    copy_sequences_to_clipboard(
                        self.value1,
                        self.value2,
                        self.sequence1.clone(),
                        self.sequence2.clone(),
                    ),
                    Message::ClipboardCopied,
                )
            }
            
            // --- Chart Generation ---
            // When the chart generation task completes, we receive a result.
            // If the result is Ok, we set the chart path to the generated file.
            // If the result is Err, we set the error message.
            // We also clear the error message if the chart was generated successfully.
            Message::ChartGenerated(result) => {
                match result {
                    Ok(path) => {
                        self.chart_path = Some(path);
                        self.error_message = String::new();
                    }
                    Err(e) => {
                        self.error_message = format!("Error generating chart: {}", e);
                        self.chart_path = None;
                    }
                }
                Command::none() // No further command needed after chart generation.
            }
            
            // When the chart saving task completes, we receive a result.
            // If the result is Ok, we set the chart saved flag to true.
            // If the result is Err, we set the error message.
            // We also clear the error message if the chart was saved successfully.
            Message::ChartSaved(result) => {
                match result {
                    Ok(()) => {
                        self.chart_saved = true;
                        self.error_message = String::new();
                    }
                    Err(e) => {
                        self.error_message = format!("Error while saving: {}", e);
                    }
                }
                Command::none() // No further command needed after chart saving.
            }
            
            // When the clipboard copy task completes, we receive a result.
            // If the result is Ok, we set the copied to clipboard flag to true.
            // If the result is Err, we set the error message.
            // We also clear the error message if the copy was successful.
            Message::ClipboardCopied(result) => {
                match result {
                    Ok(()) => {
                        self.copied_to_clipboard = true;
                        self.error_message = String::new();
                    }
                    Err(e) => {
                        self.error_message = format!("Error while copying: {}", e);
                    }
                }
                Command::none() // No further command needed after clipboard copy.
            }
            
            // When the cleanup task completes, we receive a result.
            // If the result is Ok, we ignore it (cleanup is not critical).
            // If the result is Err, we print a warning message.
            // This is done to avoid cluttering the directory with old files.
            Message::CleanupOldTempFiles(result) => {
                // On peut ignorer le résultat, car ce n'est pas critique si le nettoyage échoue
                // Mais on pourrait ajouter un log ou une notification en cas d'erreur
                if let Err(e) = result {
                    println!("Warning: Unable to delete old temporary file: {}", e);
                }
                Command::none() // No further command needed after cleanup.
            }
        }
    }

    // ==========================================================================
    //                              View Function
    // ==========================================================================
    /// This function is called to render the application's UI.
    /// It returns an `Element` that represents the entire UI.
    /// The UI is built using a combination of widgets (buttons, text inputs, etc.).
    /// The `view` function is responsible for creating the layout and appearance of the application.
    /// It uses the current state of the application to determine what to display.
    fn view(&self) -> Element<Message> {
        // Title of the application
        let title = text("Collatz Conjecture Visualizer")
            .size(28)
            .style(Color::from_rgb(0.2, 0.4, 0.8));
        
        // Input fields
        // Two text inputs for the user to enter integers.
        // The first input is required, the second is optional.
        let input_row = row![
            text("Value 1:").size(16),
            text_input("Enter an integer", &self.input1)
                .on_input(Message::Input1Changed)
                .padding(10),
            horizontal_space(Length::Fixed(20.0)),
            text("Value 2:").size(16),
            text_input("Enter an integer (optional)", &self.input2)
                .on_input(Message::Input2Changed)
                .padding(10),
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        // Button row
        // A row of buttons for user actions.
        // Each button has an action associated with it (e.g., Visualize, Randomize).
        let button_row = container(
            row![
                button("Visualize").on_press(Message::Visualize).padding(10),
                button("Randomize").on_press(Message::Randomize).padding(10),
                button("Save the graph").on_press(Message::SaveChart).padding(10),
                button("Copy the sequence").on_press(Message::CopyToClipboard).padding(10),
            ]
            .spacing(10)
            .align_items(Alignment::Center) // Centre les boutons dans la rangée
        )
        .width(Length::Fill) // Force le conteneur à prendre toute la largeur
        .center_x(); // Centre le conteneur lui-même
        
        // Status message
        // A message to display the status of the application.
        // This can be an error message, success message, or empty.
        let status_message = if !self.error_message.is_empty() {
            text(&self.error_message).style(Color::from_rgb(0.8, 0.2, 0.2))
        } else if self.chart_saved {
            text("Sequences copied to clipboard").style(Color::from_rgb(0.2, 0.8, 0.2))
        } else if self.copied_to_clipboard {
            text("Sequences copied to clipboard").style(Color::from_rgb(0.2, 0.8, 0.2))
        } else {
            text("") // Empty text if no message to display
        };
        
        // Statistics section
        // This section displays the statistics of the generated sequences.
        // If no sequences were generated, show a message indicating that.
        // If sequences were generated, display their statistics.
        // The statistics include flight time, maximum altitude, even/odd counts, and downtime.
        // The statistics are displayed in a scrollable container.
        let stats_content = if self.sequence1.is_empty() && self.sequence2.is_empty() {
            container(text("No sequence generated"))
        } else {
            let mut stats_text = String::new();
            
            // Display statistics for the first sequence
            // If the first sequence exists, display its statistics.
            // If the first value is None, it means no valid input was provided.
            if let Some(stats) = &self.stats1 {
                if let Some(value) = self.value1 {
                    stats_text.push_str(&format!("Statistics for: {}\n", value));
                    stats_text.push_str(&format!("Flight time: {} steps\n", stats.length - 1));
                    stats_text.push_str(&format!("Maximum altitude: {} (at step {})\n", 
                                               stats.max_value, stats.max_value_index));
                    stats_text.push_str(&format!("Even values: {}, Odd values: {}\n", 
                                               stats.even_count, stats.odd_count));
                    stats_text.push_str(&format!("Downtime: {} steps\n\n", stats.stopping_time));
                }
            }
            
            // Display statistics for the second sequence
            // If the second sequence exists, display its statistics.
            // If the second value is None, it means no valid input was provided.
            if let Some(stats) = &self.stats2 {
                if let Some(value) = self.value2 {
                    stats_text.push_str(&format!("Statistics for {}:\n", value));
                    stats_text.push_str(&format!("Flight time: {} steps\n", stats.length - 1));
                    stats_text.push_str(&format!("Maximum altitude: {} (at step {})\n", 
                                               stats.max_value, stats.max_value_index));
                    stats_text.push_str(&format!("Even values: {}, Odd values: {}\n", 
                                               stats.even_count, stats.odd_count));
                    stats_text.push_str(&format!("Downtime: {} steps", stats.stopping_time));
                }
            }
            
            // Create a scrollable container for the statistics text
            // This allows the user to scroll through the statistics if they are too long.
            container(
                scrollable(
                    container(text(&stats_text).size(14))
                        .padding(10)
                        .width(Length::Fill)
                )
                .height(Length::Fixed(150.0))
            )
        };
        
        // Style the statistics section
        // This section has a border and a fixed height.
        // The background color is set to a light gray.
        // The text is displayed in a scrollable container.
        // The statistics section is styled to match the application's theme.
        // The container has a border and a fixed height.
        let stats_section = container(stats_content)
            .width(Length::Fill)
            .style(|theme: &Theme| {
                container::Appearance {
                    border_width: 1.0,
                    border_color: theme.extended_palette().background.strong.color,
                    ..Default::default()
                }
            });
        
        // Chart section
        // This section displays the generated chart.
        // If a chart was generated, display it as an image.
        // If no chart was generated, display a message indicating that.
        // The chart is displayed in a container with a fixed height.
        let chart = if let Some(path) = &self.chart_path {
            container(
                image::Image::new(path.clone())
                    .width(Length::Fill)
                    .height(Length::Fixed(400.0))
                    .content_fit(iced::ContentFit::Contain)
            )
            .width(Length::Fill)
            .height(Length::Fixed(400.0))
        } else { // If no chart was generated, show a message
            container(
                text("No graph generated")
                    .width(Length::Fill)
                    .height(Length::Fixed(400.0))
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center)
            )
            .width(Length::Fill)
            .height(Length::Fixed(400.0))
            .style(|theme: &Theme| {
                container::Appearance {
                    border_width: 1.0,
                    border_color: theme.extended_palette().background.strong.color,
                    ..Default::default() // Default appearance
                }
            })
        };
        
        // Create the main content of the application
        // This includes the title, input fields, buttons, status message, and chart.
        // The content is arranged in a vertical column.
        // Each section is separated by vertical space for better readability.
        let content = column![
            title,
            vertical_space(Length::Fixed(20.0)),
            input_row,
            vertical_space(Length::Fixed(10.0)),
            button_row,
            vertical_space(Length::Fixed(10.0)),
            status_message,
            vertical_space(Length::Fixed(20.0)),
            chart,
            vertical_space(Length::Fixed(20.0)),
            text("Statistics:").size(18),
            vertical_space(Length::Fixed(5.0)),
            stats_section,
        ]
        .spacing(5)
        .padding(20)
        .max_width(800);
        
        // Create a container for the main content
        // The container has a fixed width and height, and is centered in the window.
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

// ==========================================================================
//                              Async Functions
// ==========================================================================

/// Function to clean up temporary files
/// This function checks if a file is a temporary file and deletes it.
/// It takes a path as input and returns a Result indicating success or failure.
/// The function is asynchronous, allowing it to be run in the background.
async fn cleanup_temp_file(path: String) -> Result<(), String> {
    // Checks if the file is a temporary file
    // Temporary files are identified by their name pattern.
    if path.contains("temp_collatz_") && path.ends_with(".png") {
        // Attempt to delete the temporary file
        match fs::remove_file(&path) {
            Ok(_) => Ok(()), // File deleted successfully
            Err(e) => Err(format!("Error deleting temporary file: {}", e)), // Error deleting the file
        }
    } else {
        // If the file is not a temporary file, return Ok
        Ok(())
    }
}

/// Function to clean up all temporary files
/// This function checks the current directory for temporary files and deletes them.
/// It returns a Result indicating success or failure.
async fn cleanup_all_temp_files() -> Result<(), String> {
    // Get the current directory
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir, // Current directory obtained successfully
        Err(e) => return Err(format!("Error getting current directory: {}", e)), // Error getting the directory
    };
    
    // Read the directory entries
    let entries = match fs::read_dir(current_dir) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Error reading directory: {}", e)),
    };
    
    // Iterate through the directory entries
    for entry in entries { // For each entry in the directory
        if let Ok(entry) = entry { // Check if the entry is valid
            if let Ok(file_type) = entry.file_type() { // Check if the entry is a file
                if file_type.is_file() {
                    if let Ok(file_name) = entry.file_name().into_string() { // Get the file name
                        if file_name.starts_with("temp_collatz_") && file_name.ends_with(".png") {
                            if let Err(e) = fs::remove_file(entry.path()) { // Attempt to delete the file
                                println!("Warning: Unable to delete temporary file {}: {}", file_name, e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(()) // Return success if all temporary files were processed
}

/// Asynchronously generates a chart for the Collatz sequences.
/// This function takes a path, two optional values, and two sequences.
/// It generates a chart image and saves it to the specified path.
async fn generate_chart(
    path: PathBuf, // Path to save the chart image
    value1: Option<u64>, // First value for the Collatz sequence
    value2: Option<u64>, // Second value for the Collatz sequence 
    sequence1: Vec<u64>, // First Collatz sequence
    sequence2: Vec<u64>, // Second Collatz sequence
) -> Result<String, String> {
    if sequence1.is_empty() && sequence2.is_empty() {
        return Err("No sequence to visualize".to_string());
    }
    
    // Create a temporary file for the chart
    // The file will be created in the current directory with a unique name.
    // The file will be overwritten if it already exists.
    let root = BitMapBackend::new(&path, (800, 400)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;
    
    // Determine the maximum length of the sequences
    // This is used to set the X-axis range of the chart.
    // The maximum value is used to set the Y-axis range of the chart.
    // The maximum value is determined by the highest value in both sequences.
    // If both sequences are empty, return an error.
    let max_len = sequence1.len().max(sequence2.len());
    let max_value = sequence1.iter().copied().chain(sequence2.iter().copied())
        .max().unwrap_or(1);
    
    // Create a chart builder
    // This sets up the chart's appearance and layout.
    // The chart is a Cartesian 2D chart with X and Y axes.
    // The X-axis represents the step number, and the Y-axis represents the value.
    // The chart is built using the `plotters` library.
    // The chart is drawn on the drawing area created earlier.
    let mut chart = ChartBuilder::on(&root) // Create a new chart builder
        .caption( // Set the chart caption, a string that describes the chart.
            format!(
                "Collatz Conjecture {}{}",
                value1.map_or(String::new(), |v| format!("-- {}", v)), // Handle missing value1
                value2.map_or(String::new(), |v| format!(" and {}", v)), // Append value2 if present
            ),
            ("sans-serif", 20), // Font and size for caption
        )
        .margin(10) // Margin around the chart
        .x_label_area_size(30) // Space reserved for X-axis labels
        .y_label_area_size(60) // Space reserved for Y-axis labels (adjust if numbers get large)
        // Build the coordinate system (Cartesian 2D).
        // X-axis range: 0 to max_len (number of steps).
        // Y-axis range: 0 to slightly above max_value.
        .build_cartesian_2d(0..max_len, 0..(max_value as u64 + 1))
        .map_err(|e| e.to_string())?; // Handle errors during chart building
    
    // Configure the chart's mesh (grid lines and labels).
    // The mesh is the grid that appears behind the chart.
    // The X-axis is labeled with step numbers, and the Y-axis with values.
    // The axis description style is set to a sans-serif font with size 15.
    chart.configure_mesh()
        .x_desc("Step")
        .y_desc("Value")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .map_err(|e| e.to_string())?;
    
    // Draw the first sequence
    // The first sequence is drawn in red.
    // The sequence is represented as a line on the chart.
    // Each point on the line corresponds to a step in the sequence.
    if !sequence1.is_empty() {
        chart
            .draw_series(LineSeries::new( // Draw the first sequence
                sequence1.iter().enumerate().map(|(i, &v)| (i, v)), // Enumerate the sequence
                // Convert the sequence to a series of points (x, y) for plotting.
                &RED, // Color of the line (red)
            ))
            .map_err(|e| e.to_string())? // Handle errors during drawing
            .label(format!("Sequence {}", value1.unwrap_or(0))) // Label for the first sequence
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED)); // Legend entry for the first sequence
    }
    
    // Draw the second sequence
    if !sequence2.is_empty() {
        chart
            .draw_series(LineSeries::new(
                sequence2.iter().enumerate().map(|(i, &v)| (i, v)),
                &BLUE,
            ))
            .map_err(|e| e.to_string())?
            .label(format!("Sequence {}", value2.unwrap_or(0)))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    }
    
    // Configure the legend
    // The legend is a small box that describes the colors used in the chart.
    // It shows which color corresponds to which sequence.
    // The legend is placed at the top right corner of the chart.
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .map_err(|e| e.to_string())?;
    
    // Ensure all drawing operations are finalized and written to the backend (the file).
    // This is important to ensure the chart is saved correctly.
    // The `present` method finalizes the drawing and writes the image to the file.
    // If this fails, it means there was an error writing the file.
    root.present().map_err(|e| e.to_string())?;
    
    // Return the path of the generated chart file.
    // The path is returned as a String.
    // This path can be used to access the file later (e.g., for saving or displaying).
    // The path is converted to a string using `to_string_lossy` to handle any invalid UTF-8 characters.
    // This is a safe way to convert the path to a string.
    Ok(path.to_string_lossy().to_string())
}

/// Asynchronously saves the chart by copying the temporary file to a permanent location.
/// This function takes the temporary file path and the desired target path.
/// It returns a Result indicating success or failure.
/// The target path is the filename only, not the full path.
/// The function will copy the temporary file to the target path.
/// The target path should be a valid filename, and the function will handle the full path.
/// The function is asynchronous, allowing it to be run in the background.
async fn save_chart(
    temp_path: String, // Path of the temporary chart file
    target_path: String, // Desired permanent filename (not full path yet)
) -> Result<(), String> {
    // Attempt to copy the file from the temporary path to the target path.
    fs::copy(&temp_path, &target_path)
        .map_err(|e| format!("Error copying chart file: {}", e))?;
    
    Ok(()) // If copy succeeded, return Ok.
}

/// Asynchronously formats the sequence data and copies it to the system clipboard.
/// This function takes two optional values and two sequences.
/// It returns a Result indicating success or failure.
/// The function formats the sequences into a string and sets it as the clipboard content.
/// The function is asynchronous, allowing it to be run in the background.
/// The formatted string includes the sequence data, step numbers, and values.
/// The function uses the `clipboard` crate to access the system clipboard.
async fn copy_sequences_to_clipboard(
    value1: Option<u64>,
    value2: Option<u64>,
    sequence1: Vec<u64>,
    sequence2: Vec<u64>,
) -> Result<(), String> {
    // If both sequences are empty, return an error.
    if sequence1.is_empty() && sequence2.is_empty() {
        return Err("No sequence to copy".to_string());
    }
    
    // Create a string to hold the formatted clipboard content.
    // This string will be used to set the clipboard content.
    // The string will contain the sequence data, step numbers, and values.
    // The string will be formatted to make it easy to read.
    // The string will be built using the `push_str` method to append each part.
    let mut clipboard_content = String::new();
    
    // Add the first sequence data if it exists.
    if !sequence1.is_empty() {
        // Add a header indicating which sequence it is.
        if let Some(value) = value1 {
            clipboard_content.push_str(&format!("Sequence for {}:\n", value));
        } else {
            clipboard_content.push_str("Sequence 1:\n");
        }
        
        // Append each step and value.
        // The sequence is iterated using `enumerate` to get the step number.
        // Each step is formatted as "Step X: value" and added to the clipboard content.
        // The step number is the index of the value in the sequence.
        for (i, &value) in sequence1.iter().enumerate() {
            clipboard_content.push_str(&format!("Step {}: {}\n", i, value)); // Fallback header
        }
        
        clipboard_content.push('\n'); // Add a newline for separation
    }
    
    // Add the second sequence data if it exists.
    if !sequence2.is_empty() {
        if let Some(value) = value2 {
            clipboard_content.push_str(&format!("Sequence for {}:\n", value));
        } else {
            clipboard_content.push_str("Sequence 2:\n");
        }
        
        for (i, &value) in sequence2.iter().enumerate() {
            clipboard_content.push_str(&format!("Step {}: {}\n", i, value));
        }
    }
    
    // Create a clipboard context to access the system clipboard.
    // The `clipboard` crate is used to interact with the clipboard.
    // The context is created using `ClipboardProvider::new()`.
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| format!("Clipboard initialization error: {}", e))?;
    
    // Set the clipboard content to the formatted string.
    // The `set_contents` method is used to set the clipboard content.
    // If this fails, it means there was an error accessing the clipboard.
    ctx.set_contents(clipboard_content)
        .map_err(|e| format!("Error while copying: {}", e))?;
    
    Ok(()) // If everything succeeded, return Ok.
}

// ==========================================================================
//                              Main Function
// ==========================================================================

fn main() -> iced::Result {
    // Attempt to clean up any leftover temporary files.
    // This is done to ensure that the application starts with a clean slate.
    // The cleanup function is called asynchronously, but we use `block_on` to wait for it to finish.
    // This is necessary because the main function cannot be async.
    let _ = futures::executor::block_on(cleanup_all_temp_files());
    
    // Run the application with the default settings.
    // The `CollatzApp` is the main application struct that implements the Iced framework.
    // The `run` method starts the application and enters the event loop.
    // The `Settings::default()` provides the default settings for the application.
    // The application will run until it is closed by the user.
    CollatzApp::run(Settings::default())
}
