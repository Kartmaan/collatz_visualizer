// Fichier collatz.rs
// Implémentation de l'algorithme de la conjecture de Syracuse (Collatz)

/// Calcule la suite de Collatz pour un nombre donné
/// 
/// La règle de la conjecture de Syracuse est:
/// - Si n est pair, le terme suivant est n/2
/// - Si n est impair, le terme suivant est 3n+1
/// 
/// La suite s'arrête lorsque la valeur 1 est atteinte
pub fn generate_sequence(start: u64) -> Vec<u64> {
    if start == 0 {
        return vec![0]; // Cas spécial pour 0
    }

    let mut sequence = Vec::new();
    let mut current = start;
    
    sequence.push(current);
    
    while current != 1 {
        if current % 2 == 0 {
            // Si n est pair
            current = current / 2;
        } else {
            // Si n est impair
            // Vérification pour éviter les dépassements d'entiers
            if current > (u64::MAX - 1) / 3 {
                // En cas de risque de dépassement, on arrête la séquence
                sequence.push(current);
                break;
            }
            current = 3 * current + 1;
        }
        sequence.push(current);
    }
    
    sequence
}

/// Calcule des statistiques sur une suite de Collatz
pub struct CollatzStats {
    pub length: usize,           // Longueur de la séquence (temps de vol total)
    pub max_value: u64,          // Valeur maximale atteinte (altitude)
    pub max_value_index: usize,  // Position de la valeur maximale
    pub even_count: usize,       // Nombre de valeurs paires
    pub odd_count: usize,        // Nombre de valeurs impaires
    pub stopping_time: usize,    // Temps d'arrêt (nombre d'étapes pour atteindre une valeur < start)
}

/// Calcule des statistiques sur une suite de Collatz
pub fn calculate_stats(sequence: &[u64]) -> CollatzStats {
    if sequence.is_empty() {
        return CollatzStats {
            length: 0,
            max_value: 0,
            max_value_index: 0,
            even_count: 0,
            odd_count: 0,
            stopping_time: 0,
        };
    }

    let start_value = sequence[0];
    let length = sequence.len();
    
    // Trouver la valeur maximale et son index
    let (max_value_index, max_value) = sequence.iter()
        .enumerate()
        .max_by_key(|&(_, &value)| value)
        .unwrap_or((0, &0));
    
    // Compter les valeurs paires et impaires
    let even_count = sequence.iter().filter(|&&n| n % 2 == 0).count();
    let odd_count = length - even_count;
    
    // Calculer le temps d'arrêt (nombre d'étapes pour atteindre une valeur < start)
    let stopping_time = sequence.iter()
        .enumerate()
        .skip(1) // Sauter la première valeur (start)
        .find(|&(_, &value)| value < start_value)
        .map(|(index, _)| index)
        .unwrap_or(length - 1);
    
    CollatzStats {
        length,
        max_value: *max_value,
        max_value_index,
        even_count,
        odd_count,
        stopping_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sequence() {
        // Test avec n = 6
        let sequence = generate_sequence(6);
        assert_eq!(sequence, vec![6, 3, 10, 5, 16, 8, 4, 2, 1]);
        
        // Test avec n = 1
        let sequence = generate_sequence(1);
        assert_eq!(sequence, vec![1]);
    }

    #[test]
    fn test_calculate_stats() {
        // Test avec n = 6
        let sequence = generate_sequence(6);
        let stats = calculate_stats(&sequence);
        
        assert_eq!(stats.length, 9);
        assert_eq!(stats.max_value, 16);
        assert_eq!(stats.max_value_index, 4);
        assert_eq!(stats.even_count, 6);
        assert_eq!(stats.odd_count, 3);
        assert_eq!(stats.stopping_time, 1); // 3 < 6 après 1 étape
    }
}
