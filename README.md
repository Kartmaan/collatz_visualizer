# Application de Visualisation de la Conjecture de Syracuse (Collatz)

Cette application permet de visualiser graphiquement les suites de la conjecture de Syracuse (également connue sous le nom de conjecture de Collatz) pour un ou deux entiers donnés.

## La Conjecture de Syracuse

La conjecture de Syracuse est définie par la règle suivante :
- Si n est pair, le terme suivant est n/2
- Si n est impair, le terme suivant est 3n+1

La suite s'arrête lorsque la valeur 1 est atteinte.

## Fonctionnalités

- Visualisation graphique des suites pour un ou deux entiers
- Affichage de statistiques détaillées :
  - Temps de vol (nombre d'étapes)
  - Altitude maximale (valeur maximale atteinte)
  - Nombre de valeurs paires/impaires
  - Temps d'arrêt
- Génération de valeurs aléatoires
- Enregistrement de l'image du graphique
- Copie des suites dans le presse-papier

## Installation

### Prérequis

- Rust et Cargo (https://www.rust-lang.org/tools/install)

### Installation

1. Décompressez l'archive zip
2. Ouvrez un terminal dans le dossier décompressé
3. Exécutez la commande suivante pour compiler et lancer l'application :

```bash
cargo run
```

Pour créer une version optimisée :

```bash
cargo build --release
```

L'exécutable se trouvera dans le dossier `target/release/`.

## Utilisation

1. Entrez un ou deux nombres entiers dans les champs de saisie
2. Cliquez sur "Visualiser" pour afficher le graphique
3. Utilisez "Randomize" pour générer des valeurs aléatoires
4. Utilisez "Enregistrer" pour sauvegarder l'image du graphique
5. Utilisez "Copier" pour copier les suites dans le presse-papier

## Structure du code

- `src/main.rs` : Interface utilisateur et logique principale de l'application
- `src/collatz.rs` : Implémentation de l'algorithme de la conjecture de Syracuse

## Dépendances

- iced : Interface utilisateur graphique
- plotters : Visualisation graphique
- rand : Génération de nombres aléatoires
- clipboard : Accès au presse-papier
- chrono : Gestion des dates et heures
- image : Manipulation d'images

## Licence

Ce projet est distribué sous licence MIT.
