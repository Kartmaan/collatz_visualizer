// Fichier main.rs
// Application principale pour visualiser les suites de Collatz

mod collatz;

use iced::{
    widget::{
        button, column, container, row, text, text_input, vertical_space, horizontal_space,
        scrollable, image,
    },
    executor, Application, Command, Element, Length, Settings, Theme, Color, Alignment,
};
use plotters::prelude::*;
use plotters::style::Color as PlottersColor;
use rand::Rng;
use std::path::PathBuf;
use clipboard::{ClipboardContext, ClipboardProvider};
use chrono::Local;
use std::fs;

// Structure principale de l'application
pub struct CollatzApp {
    // Champs de saisie
    input1: String,
    input2: String,
    
    // Valeurs calculées
    value1: Option<u64>,
    value2: Option<u64>,
    
    // Séquences calculées
    sequence1: Vec<u64>,
    sequence2: Vec<u64>,
    
    // Statistiques
    stats1: Option<collatz::CollatzStats>,
    stats2: Option<collatz::CollatzStats>,
    
    // État de l'application
    error_message: String,
    chart_saved: bool,
    copied_to_clipboard: bool,
    
    // Chemin de l'image du graphique
    chart_path: Option<String>,
}

// Messages pour l'application
#[derive(Debug, Clone)]
pub enum Message {
    Input1Changed(String),
    Input2Changed(String),
    Visualize,
    Randomize,
    SaveChart,
    CopyToClipboard,
    ChartGenerated(Result<String, String>),
    ChartSaved(Result<(), String>),
    ClipboardCopied(Result<(), String>),
    CleanupOldTempFiles(Result<(), String>),
}

impl Application for CollatzApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                input1: String::new(),
                input2: String::new(),
                value1: None,
                value2: None,
                sequence1: Vec::new(),
                sequence2: Vec::new(),
                stats1: None,
                stats2: None,
                error_message: String::new(),
                chart_saved: false,
                copied_to_clipboard: false,
                chart_path: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let mut title = String::from("Visualisation de la Conjecture de Syracuse");
        
        if let Some(v1) = self.value1 {
            title.push_str(&format!(" - {}", v1));
            
            if let Some(v2) = self.value2 {
                title.push_str(&format!(" et {}", v2));
            }
        }
        
        title
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Input1Changed(value) => {
                self.input1 = value;
                Command::none()
            }
            
            Message::Input2Changed(value) => {
                self.input2 = value;
                Command::none()
            }
            
            Message::Visualize => {
                self.error_message = String::new();
                self.chart_saved = false;
                self.copied_to_clipboard = false;
                
                // Traiter la première entrée
                match self.input1.trim().parse::<u64>() {
                    Ok(value) => {
                        if value == 0 {
                            self.error_message = "La valeur doit être supérieure à 0".to_string();
                            return Command::none();
                        }
                        
                        self.value1 = Some(value);
                        self.sequence1 = collatz::generate_sequence(value);
                        self.stats1 = Some(collatz::calculate_stats(&self.sequence1));
                    }
                    Err(_) => {
                        if !self.input1.trim().is_empty() {
                            self.error_message = "Première valeur invalide".to_string();
                        } else {
                            self.value1 = None;
                            self.sequence1.clear();
                            self.stats1 = None;
                        }
                    }
                }
                
                // Traiter la deuxième entrée
                match self.input2.trim().parse::<u64>() {
                    Ok(value) => {
                        if value == 0 {
                            self.error_message = "La valeur doit être supérieure à 0".to_string();
                            return Command::none();
                        }
                        
                        self.value2 = Some(value);
                        self.sequence2 = collatz::generate_sequence(value);
                        self.stats2 = Some(collatz::calculate_stats(&self.sequence2));
                    }
                    Err(_) => {
                        if !self.input2.trim().is_empty() {
                            self.error_message = "Deuxième valeur invalide".to_string();
                        } else {
                            self.value2 = None;
                            self.sequence2.clear();
                            self.stats2 = None;
                        }
                    }
                }
                
                // Si au moins une séquence a été générée, créer le graphique
                if !self.sequence1.is_empty() || !self.sequence2.is_empty() {
                    // D'abord, nettoyer l'ancien fichier temporaire s'il existe
                    let cleanup_command = if let Some(old_path) = &self.chart_path {
                        Command::perform(
                            cleanup_temp_file(old_path.clone()),
                            Message::CleanupOldTempFiles,
                        )
                    } else {
                        Command::none()
                    };
                    
                    // Générer un nom de fichier temporaire pour le graphique
                    let now = Local::now();
                    let filename = format!("temp_collatz_{}.png", now.format("%Y%m%d_%H%M%S"));
                    
                    // Ensuite, générer le nouveau graphique
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
                    
                    // Exécuter les deux commandes en séquence
                    Command::batch(vec![cleanup_command, generate_command])
                } else {
                    Command::none()
                }
            }
            
            Message::Randomize => {
                let mut rng = rand::thread_rng();
                
                // Générer deux nombres aléatoires entre 1 et 1000
                let random1 = rng.gen_range(1..=1000);
                let random2 = rng.gen_range(1..=1000);
                
                self.input1 = random1.to_string();
                self.input2 = random2.to_string();
                
                // Appeler Visualize pour mettre à jour les séquences
                self.update(Message::Visualize)
            }
            
            Message::SaveChart => {
                if self.sequence1.is_empty() && self.sequence2.is_empty() {
                    self.error_message = "Aucune séquence à enregistrer".to_string();
                    return Command::none();
                }
                
                if self.chart_path.is_none() {
                    self.error_message = "Aucun graphique à enregistrer".to_string();
                    return Command::none();
                }
                
                self.chart_saved = false;
                
                // Créer un nom de fichier avec la date et l'heure actuelles
                let now = Local::now();
                let filename = format!("collatz_{}.png", now.format("%Y%m%d_%H%M%S"));
                
                // Copier le fichier temporaire vers le fichier final
                Command::perform(
                    save_chart(
                        self.chart_path.clone().unwrap(),
                        filename,
                    ),
                    Message::ChartSaved,
                )
            }
            
            Message::CopyToClipboard => {
                if self.sequence1.is_empty() && self.sequence2.is_empty() {
                    self.error_message = "Aucune séquence à copier".to_string();
                    return Command::none();
                }
                
                self.copied_to_clipboard = false;
                
                // Créer une commande pour copier les séquences dans le presse-papier
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
            
            Message::ChartGenerated(result) => {
                match result {
                    Ok(path) => {
                        self.chart_path = Some(path);
                        self.error_message = String::new();
                    }
                    Err(e) => {
                        self.error_message = format!("Erreur lors de la génération du graphique: {}", e);
                        self.chart_path = None;
                    }
                }
                Command::none()
            }
            
            Message::ChartSaved(result) => {
                match result {
                    Ok(()) => {
                        self.chart_saved = true;
                        self.error_message = String::new();
                    }
                    Err(e) => {
                        self.error_message = format!("Erreur lors de l'enregistrement: {}", e);
                    }
                }
                Command::none()
            }
            
            Message::ClipboardCopied(result) => {
                match result {
                    Ok(()) => {
                        self.copied_to_clipboard = true;
                        self.error_message = String::new();
                    }
                    Err(e) => {
                        self.error_message = format!("Erreur lors de la copie: {}", e);
                    }
                }
                Command::none()
            }
            
            Message::CleanupOldTempFiles(result) => {
                // On peut ignorer le résultat, car ce n'est pas critique si le nettoyage échoue
                // Mais on pourrait ajouter un log ou une notification en cas d'erreur
                if let Err(e) = result {
                    println!("Avertissement: Impossible de supprimer l'ancien fichier temporaire: {}", e);
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Titre de l'application
        let title = text("Visualisation de la Conjecture de Syracuse")
            .size(28)
            .style(Color::from_rgb(0.2, 0.4, 0.8));
        
        // Champs de saisie
        let input_row = row![
            text("Valeur 1:").size(16),
            text_input("Entrez un nombre entier", &self.input1)
                .on_input(Message::Input1Changed)
                .padding(10),
            horizontal_space(Length::Fixed(20.0)),
            text("Valeur 2:").size(16),
            text_input("Entrez un nombre entier (optionnel)", &self.input2)
                .on_input(Message::Input2Changed)
                .padding(10),
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        // Boutons d'action
        let button_row = row![
            button("Visualiser").on_press(Message::Visualize).padding(10),
            button("Randomize").on_press(Message::Randomize).padding(10),
            button("Enregistrer").on_press(Message::SaveChart).padding(10),
            button("Copier").on_press(Message::CopyToClipboard).padding(10),
        ]
        .spacing(10);
        
        // Message d'erreur ou de confirmation
        let status_message = if !self.error_message.is_empty() {
            text(&self.error_message).style(Color::from_rgb(0.8, 0.2, 0.2))
        } else if self.chart_saved {
            text("Graphique enregistré avec succès!").style(Color::from_rgb(0.2, 0.8, 0.2))
        } else if self.copied_to_clipboard {
            text("Séquences copiées dans le presse-papier!").style(Color::from_rgb(0.2, 0.8, 0.2))
        } else {
            text("")
        };
        
        // Zone de statistiques
        let stats_content = if self.sequence1.is_empty() && self.sequence2.is_empty() {
            container(text("Aucune séquence générée"))
        } else {
            let mut stats_text = String::new();
            
            if let Some(stats) = &self.stats1 {
                if let Some(value) = self.value1 {
                    stats_text.push_str(&format!("Statistiques pour {}:\n", value));
                    stats_text.push_str(&format!("Temps de vol: {} étapes\n", stats.length - 1));
                    stats_text.push_str(&format!("Altitude maximale: {} (à l'étape {})\n", 
                                               stats.max_value, stats.max_value_index));
                    stats_text.push_str(&format!("Valeurs paires: {}, Valeurs impaires: {}\n", 
                                               stats.even_count, stats.odd_count));
                    stats_text.push_str(&format!("Temps d'arrêt: {} étapes\n\n", stats.stopping_time));
                }
            }
            
            if let Some(stats) = &self.stats2 {
                if let Some(value) = self.value2 {
                    stats_text.push_str(&format!("Statistiques pour {}:\n", value));
                    stats_text.push_str(&format!("Temps de vol: {} étapes\n", stats.length - 1));
                    stats_text.push_str(&format!("Altitude maximale: {} (à l'étape {})\n", 
                                               stats.max_value, stats.max_value_index));
                    stats_text.push_str(&format!("Valeurs paires: {}, Valeurs impaires: {}\n", 
                                               stats.even_count, stats.odd_count));
                    stats_text.push_str(&format!("Temps d'arrêt: {} étapes", stats.stopping_time));
                }
            }
            
            container(
                scrollable(
                    container(text(&stats_text).size(14))
                        .padding(10)
                        .width(Length::Fill)
                )
                .height(Length::Fixed(150.0))
            )
        };
        
        let stats_section = container(stats_content)
            .width(Length::Fill)
            .style(|theme: &Theme| {
                container::Appearance {
                    border_width: 1.0,
                    border_color: theme.extended_palette().background.strong.color,
                    ..Default::default()
                }
            });
        
        // Graphique
        let chart = if let Some(path) = &self.chart_path {
            container(
                image::Image::new(path.clone())
                    .width(Length::Fill)
                    .height(Length::Fixed(400.0))
            )
            .width(Length::Fill)
            .height(Length::Fixed(400.0))
        } else {
            container(
                text("Aucun graphique généré")
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
                    ..Default::default()
                }
            })
        };
        
        // Mise en page principale
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
            text("Statistiques:").size(18),
            vertical_space(Length::Fixed(5.0)),
            stats_section,
        ]
        .spacing(5)
        .padding(20)
        .max_width(800);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

// Fonction pour nettoyer les fichiers temporaires
async fn cleanup_temp_file(path: String) -> Result<(), String> {
    // Vérifier si le fichier existe et s'il s'agit d'un fichier temporaire
    if path.contains("temp_collatz_") && path.ends_with(".png") {
        // Supprimer le fichier
        match fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Erreur lors de la suppression du fichier temporaire: {}", e)),
        }
    } else {
        // Si ce n'est pas un fichier temporaire, ne rien faire
        Ok(())
    }
}

// Fonction pour nettoyer tous les fichiers temporaires du dossier courant
async fn cleanup_all_temp_files() -> Result<(), String> {
    // Obtenir le répertoire courant
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(format!("Erreur lors de l'obtention du répertoire courant: {}", e)),
    };
    
    // Lire le contenu du répertoire
    let entries = match fs::read_dir(current_dir) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Erreur lors de la lecture du répertoire: {}", e)),
    };
    
    // Parcourir les entrées et supprimer les fichiers temporaires
    for entry in entries {
        if let Ok(entry) = entry {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.starts_with("temp_collatz_") && file_name.ends_with(".png") {
                            if let Err(e) = fs::remove_file(entry.path()) {
                                println!("Avertissement: Impossible de supprimer le fichier temporaire {}: {}", file_name, e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

// Fonction pour générer le graphique
async fn generate_chart(
    path: PathBuf,
    value1: Option<u64>,
    value2: Option<u64>,
    sequence1: Vec<u64>,
    sequence2: Vec<u64>,
) -> Result<String, String> {
    if sequence1.is_empty() && sequence2.is_empty() {
        return Err("Aucune séquence à visualiser".to_string());
    }
    
    // Créer une image pour le graphique
    let root = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;
    
    // Déterminer les limites du graphique
    let max_len = sequence1.len().max(sequence2.len());
    let max_value = sequence1.iter().copied().chain(sequence2.iter().copied())
        .max().unwrap_or(1);
    
    // Créer le graphique
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "Conjecture de Syracuse{}{}",
                value1.map_or(String::new(), |v| format!(" - {}", v)),
                value2.map_or(String::new(), |v| format!(" et {}", v)),
            ),
            ("sans-serif", 20),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0..max_len, 0..(max_value as u64 + 1))
        .map_err(|e| e.to_string())?;
    
    chart.configure_mesh()
        .x_desc("Étape")
        .y_desc("Valeur")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .map_err(|e| e.to_string())?;
    
    // Tracer la première séquence
    if !sequence1.is_empty() {
        chart
            .draw_series(LineSeries::new(
                sequence1.iter().enumerate().map(|(i, &v)| (i, v)),
                &RED,
            ))
            .map_err(|e| e.to_string())?
            .label(format!("Séquence {}", value1.unwrap_or(0)))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    }
    
    // Tracer la deuxième séquence
    if !sequence2.is_empty() {
        chart
            .draw_series(LineSeries::new(
                sequence2.iter().enumerate().map(|(i, &v)| (i, v)),
                &BLUE,
            ))
            .map_err(|e| e.to_string())?
            .label(format!("Séquence {}", value2.unwrap_or(0)))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    }
    
    // Ajouter la légende
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .map_err(|e| e.to_string())?;
    
    // Finaliser le graphique
    root.present().map_err(|e| e.to_string())?;
    
    Ok(path.to_string_lossy().to_string())
}

// Fonction pour enregistrer le graphique
async fn save_chart(
    temp_path: String,
    target_path: String,
) -> Result<(), String> {
    // Copier le fichier temporaire vers le fichier final
    fs::copy(&temp_path, &target_path)
        .map_err(|e| format!("Erreur lors de la copie du fichier: {}", e))?;
    
    Ok(())
}

// Fonction pour copier les séquences dans le presse-papier
async fn copy_sequences_to_clipboard(
    value1: Option<u64>,
    value2: Option<u64>,
    sequence1: Vec<u64>,
    sequence2: Vec<u64>,
) -> Result<(), String> {
    if sequence1.is_empty() && sequence2.is_empty() {
        return Err("Aucune séquence à copier".to_string());
    }
    
    let mut clipboard_content = String::new();
    
    // Ajouter la première séquence
    if !sequence1.is_empty() {
        if let Some(value) = value1 {
            clipboard_content.push_str(&format!("Séquence pour {}:\n", value));
        } else {
            clipboard_content.push_str("Séquence 1:\n");
        }
        
        for (i, &value) in sequence1.iter().enumerate() {
            clipboard_content.push_str(&format!("Étape {}: {}\n", i, value));
        }
        
        clipboard_content.push('\n');
    }
    
    // Ajouter la deuxième séquence
    if !sequence2.is_empty() {
        if let Some(value) = value2 {
            clipboard_content.push_str(&format!("Séquence pour {}:\n", value));
        } else {
            clipboard_content.push_str("Séquence 2:\n");
        }
        
        for (i, &value) in sequence2.iter().enumerate() {
            clipboard_content.push_str(&format!("Étape {}: {}\n", i, value));
        }
    }
    
    // Copier dans le presse-papier
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| format!("Erreur d'initialisation du presse-papier: {}", e))?;
    
    ctx.set_contents(clipboard_content)
        .map_err(|e| format!("Erreur lors de la copie: {}", e))?;
    
    Ok(())
}

fn main() -> iced::Result {
    // Au démarrage de l'application, nettoyer tous les fichiers temporaires
    // qui pourraient être restés d'une exécution précédente
    let _ = futures::executor::block_on(cleanup_all_temp_files());
    
    CollatzApp::run(Settings::default())
}
