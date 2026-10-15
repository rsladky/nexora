//! Décodage des sorties du détecteur de paume et NMS pondérée.

use super::anchors::PALM_INPUT;

/// Paume détectée, coordonnées normalisées 0..1 dans l'image letterboxée (192x192).
#[derive(Debug, Clone, PartialEq)]
pub struct PalmDetection {
    pub score: f32,
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    /// 7 points-clés : 0 = poignet, 2 = base du majeur.
    pub keypoints: [[f32; 2]; 7],
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x.clamp(-100.0, 100.0)).exp())
}

/// `regressors` : `[2016 * 18]` (cx, cy, w, h, 7 x (x, y)), `scores` : `[2016]` (logits).
pub fn decode(
    regressors: &[f32],
    scores: &[f32],
    anchors: &[[f32; 2]],
    threshold: f32,
) -> Vec<PalmDetection> {
    let s = PALM_INPUT as f32;
    let mut out = Vec::new();
    for (i, anchor) in anchors.iter().enumerate() {
        let score = sigmoid(scores[i]);
        if score < threshold {
            continue;
        }
        let r = &regressors[i * 18..(i + 1) * 18];
        let mut keypoints = [[0.0; 2]; 7];
        for (k, kp) in keypoints.iter_mut().enumerate() {
            *kp = [r[4 + 2 * k] / s + anchor[0], r[5 + 2 * k] / s + anchor[1]];
        }
        out.push(PalmDetection {
            score,
            cx: r[0] / s + anchor[0],
            cy: r[1] / s + anchor[1],
            w: r[2] / s,
            h: r[3] / s,
            keypoints,
        });
    }
    out
}

fn iou(a: &PalmDetection, b: &PalmDetection) -> f32 {
    let (ax0, ax1) = (a.cx - a.w / 2.0, a.cx + a.w / 2.0);
    let (ay0, ay1) = (a.cy - a.h / 2.0, a.cy + a.h / 2.0);
    let (bx0, bx1) = (b.cx - b.w / 2.0, b.cx + b.w / 2.0);
    let (by0, by1) = (b.cy - b.h / 2.0, b.cy + b.h / 2.0);
    let iw = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
    let ih = (ay1.min(by1) - ay0.max(by0)).max(0.0);
    let inter = iw * ih;
    let union = a.w * a.h + b.w * b.h - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}

/// NMS pondérée : les boîtes qui se recouvrent (IoU > `iou_thr`) sont fusionnées
/// en moyenne pondérée par leur score. Résultat trié par score décroissant.
pub fn weighted_nms(mut dets: Vec<PalmDetection>, iou_thr: f32) -> Vec<PalmDetection> {
    dets.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut out = Vec::new();
    while !dets.is_empty() {
        let best = dets.remove(0);
        let mut group = vec![best.clone()];
        dets.retain(|d| {
            if iou(&best, d) > iou_thr {
                group.push(d.clone());
                false
            } else {
                true
            }
        });
        let total: f32 = group.iter().map(|d| d.score).sum();
        let mut merged = best;
        let wavg = |f: &dyn Fn(&PalmDetection) -> f32| -> f32 {
            group.iter().map(|d| f(d) * d.score).sum::<f32>() / total
        };
        merged.cx = wavg(&|d| d.cx);
        merged.cy = wavg(&|d| d.cy);
        merged.w = wavg(&|d| d.w);
        merged.h = wavg(&|d| d.h);
        for k in 0..7 {
            merged.keypoints[k] = [wavg(&|d| d.keypoints[k][0]), wavg(&|d| d.keypoints[k][1])];
        }
        out.push(merged);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn det(score: f32, cx: f32, cy: f32) -> PalmDetection {
        PalmDetection {
            score,
            cx,
            cy,
            w: 0.2,
            h: 0.2,
            keypoints: [[cx, cy]; 7],
        }
    }

    #[test]
    fn decode_single_anchor() {
        let anchors = vec![[0.5, 0.5], [0.25, 0.25]];
        let mut reg = vec![0.0; 2 * 18];
        reg[0] = 19.2; // +0.1 en x
        reg[2] = 38.4; // w = 0.2
        reg[4] = 96.0; // keypoint 0 : +0.5 en x
        let scores = [10.0, -10.0];
        let d = decode(&reg, &scores, &anchors, 0.5);
        assert_eq!(d.len(), 1);
        assert!((d[0].cx - 0.6).abs() < 1e-5);
        assert!((d[0].w - 0.2).abs() < 1e-5);
        assert!((d[0].keypoints[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn nms_merges_overlap_and_keeps_disjoint() {
        let merged = weighted_nms(vec![det(0.9, 0.5, 0.5), det(0.8, 0.52, 0.5)], 0.3);
        assert_eq!(merged.len(), 1);
        assert!(merged[0].cx > 0.5 && merged[0].cx < 0.52);
        let apart = weighted_nms(vec![det(0.9, 0.2, 0.2), det(0.8, 0.8, 0.8)], 0.3);
        assert_eq!(apart.len(), 2);
        assert!(apart[0].score >= apart[1].score);
    }
}
