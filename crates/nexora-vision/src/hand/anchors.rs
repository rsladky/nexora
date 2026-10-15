//! Ancres SSD du détecteur de paume MediaPipe (entrée 192x192).

/// Taille d'entrée du détecteur de paume.
pub const PALM_INPUT: usize = 192;
/// Nombre d'ancres : 24*24*2 + 12*12*6.
pub const NUM_ANCHORS: usize = 2016;

/// Centres des ancres (x, y) normalisés 0..1, dans l'ordre des sorties du modèle.
/// Strides 8/16/16/16 : les couches de même stride sont fusionnées (2 + 2 + 2 ancres
/// par cellule en 12x12, soit 6). Les ancres ont une taille fixe de 1.0.
pub fn generate_anchors() -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(NUM_ANCHORS);
    for (cells, per_cell) in [(24usize, 2usize), (12, 6)] {
        for y in 0..cells {
            for x in 0..cells {
                for _ in 0..per_cell {
                    out.push([
                        (x as f32 + 0.5) / cells as f32,
                        (y as f32 + 0.5) / cells as f32,
                    ]);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_count_and_first_center() {
        let a = generate_anchors();
        assert_eq!(a.len(), NUM_ANCHORS);
        assert!((a[0][0] - 1.0 / 48.0).abs() < 1e-6);
        assert_eq!(a[0], a[1]);
        // Première ancre de la grille 12x12.
        assert!((a[1152][0] - 1.0 / 24.0).abs() < 1e-6);
    }
}
