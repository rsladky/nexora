//! Filtre One-Euro (lissage adaptatif) appliqué aux 21 landmarks.

use std::f32::consts::PI;

use crate::Landmarks;

#[derive(Debug, Clone)]
pub struct OneEuro {
    min_cutoff: f32,
    beta: f32,
    d_cutoff: f32,
    prev: Option<(f32, f32)>, // (valeur filtrée, dérivée filtrée)
}

fn alpha(cutoff: f32, dt: f32) -> f32 {
    let tau = 1.0 / (2.0 * PI * cutoff);
    1.0 / (1.0 + tau / dt)
}

impl OneEuro {
    pub fn new(min_cutoff: f32, beta: f32) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff: 1.0,
            prev: None,
        }
    }

    pub fn reset(&mut self) {
        self.prev = None;
    }

    /// `dt` en secondes depuis l'échantillon précédent.
    pub fn filter(&mut self, x: f32, dt: f32) -> f32 {
        let dt = dt.max(1e-3);
        let Some((px, pd)) = self.prev else {
            self.prev = Some((x, 0.0));
            return x;
        };
        let d = (x - px) / dt;
        let d_hat = pd + alpha(self.d_cutoff, dt) * (d - pd);
        let cutoff = self.min_cutoff + self.beta * d_hat.abs();
        let x_hat = px + alpha(cutoff, dt) * (x - px);
        self.prev = Some((x_hat, d_hat));
        x_hat
    }
}

/// 63 filtres (21 points x 3 axes). Coordonnées normalisées 0..1, d'où un `beta` élevé.
pub struct LandmarkFilter {
    filters: Vec<OneEuro>,
}

impl LandmarkFilter {
    pub fn new(min_cutoff: f32, beta: f32) -> Self {
        Self {
            filters: vec![OneEuro::new(min_cutoff, beta); 63],
        }
    }

    pub fn reset(&mut self) {
        self.filters.iter_mut().for_each(OneEuro::reset);
    }

    pub fn apply(&mut self, l: &mut Landmarks, dt: f32) {
        for (i, p) in l.iter_mut().enumerate() {
            for (a, v) in p.iter_mut().enumerate() {
                *v = self.filters[3 * i + a].filter(*v, dt);
            }
        }
    }
}

impl Default for LandmarkFilter {
    fn default() -> Self {
        Self::new(1.5, 20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_input_stays_constant() {
        let mut f = OneEuro::new(1.0, 10.0);
        for _ in 0..20 {
            assert!((f.filter(0.4, 0.033) - 0.4).abs() < 1e-6);
        }
    }

    #[test]
    fn attenuates_jitter_but_follows_fast_motion() {
        let mut f = OneEuro::new(1.0, 20.0);
        f.filter(0.5, 0.033);
        let jitter = f.filter(0.502, 0.033);
        assert!((jitter - 0.5).abs() < 0.002);
        // Mouvement rapide : le filtre rattrape presque la cible.
        let mut g = OneEuro::new(1.0, 20.0);
        g.filter(0.0, 0.033);
        let mut v = 0.0;
        for _ in 0..5 {
            v = g.filter(1.0, 0.033);
        }
        assert!(v > 0.8);
    }
}
