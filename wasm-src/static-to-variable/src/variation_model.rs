//! OpenType variation-model construction.
//!
//! This follows the region and master-support rules defined by the OpenType
//! specification and FontTools' `VariationModel`. It is kept local so the
//! browser engine has no Python dependency or source-format dependency.

use std::cmp::Ordering;

use crate::error::BuildError;

const EPSILON: f64 = 1e-9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tent {
    pub minimum: f64,
    pub peak: f64,
    pub maximum: f64,
}

impl Tent {
    const ZERO: Self = Self {
        minimum: 0.0,
        peak: 0.0,
        maximum: 0.0,
    };

    fn from_peak(peak: f64, minimum: f64, maximum: f64) -> Self {
        if approximately_zero(peak) {
            Self::ZERO
        } else if peak > 0.0 {
            Self {
                minimum: 0.0,
                peak,
                maximum,
            }
        } else {
            Self {
                minimum,
                peak,
                maximum: 0.0,
            }
        }
    }

    fn is_zero(self) -> bool {
        approximately_zero(self.minimum)
            && approximately_zero(self.peak)
            && approximately_zero(self.maximum)
    }

    fn scalar_at(self, value: f64) -> f64 {
        if self.is_zero() || approximately_equal(value, self.peak) {
            return 1.0;
        }
        if self.minimum > self.peak
            || self.peak > self.maximum
            || (self.minimum < 0.0 && self.maximum > 0.0)
            || value <= self.minimum
            || value >= self.maximum
        {
            return 0.0;
        }
        if value < self.peak {
            (value - self.minimum) / (self.peak - self.minimum)
        } else {
            (value - self.maximum) / (self.peak - self.maximum)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Support {
    pub tents: Vec<Tent>,
}

impl Support {
    pub fn is_default(&self) -> bool {
        self.tents.iter().all(|tent| tent.is_zero())
    }

    fn scalar_at(&self, location: &[f64]) -> f64 {
        self.tents
            .iter()
            .zip(location)
            .map(|(tent, coordinate)| tent.scalar_at(*coordinate))
            .product()
    }
}

#[derive(Debug)]
pub struct VariationModel {
    order: Vec<usize>,
    supports: Vec<Support>,
    weights: Vec<Vec<(usize, f64)>>,
}

impl VariationModel {
    pub fn new(locations: Vec<Vec<f64>>) -> Result<Self, BuildError> {
        if locations.is_empty() {
            return Err(BuildError::MissingDefault);
        }
        let axis_count = locations[0].len();
        if locations
            .iter()
            .any(|location| location.len() != axis_count)
        {
            return Err(BuildError::MissingDefault);
        }
        if has_duplicate_locations(&locations) {
            return Err(BuildError::DuplicateLocation);
        }

        let mut order: Vec<usize> = (0..locations.len()).collect();
        let on_axis_points = axis_points(&locations, axis_count);
        order.sort_by(|left, right| {
            location_sort_key(&locations[*left], &on_axis_points)
                .partial_cmp(&location_sort_key(&locations[*right], &on_axis_points))
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.cmp(right))
        });

        let ordered_locations: Vec<Vec<f64>> = order
            .iter()
            .map(|index| locations[*index].clone())
            .collect();
        if !ordered_locations[0]
            .iter()
            .all(|coordinate| approximately_zero(*coordinate))
        {
            return Err(BuildError::MissingDefault);
        }

        let regions = regions_for(&ordered_locations, axis_count);
        let supports = master_supports(&regions);
        let weights = delta_weights(&ordered_locations, &supports);
        Ok(Self {
            order,
            supports,
            weights,
        })
    }

    /// Converts absolute master values to OpenType deltas. The callback returns
    /// one fixed-size coordinate vector for a source-master index.
    pub fn deltas<T: DeltaValue>(
        &self,
        values: &[Vec<T>],
    ) -> Result<Vec<(usize, Support, Vec<T>)>, BuildError> {
        if values.len() != self.order.len() {
            return Err(BuildError::MissingDefault);
        }
        let point_count = values.first().map_or(0, Vec::len);
        if values.iter().any(|value| value.len() != point_count) {
            return Err(BuildError::MissingDefault);
        }

        let mut computed: Vec<Vec<T>> = Vec::with_capacity(self.order.len());
        for (model_index, source_index) in self.order.iter().enumerate() {
            let mut delta = values[*source_index].clone();
            for (influencer, scalar) in &self.weights[model_index] {
                for (value, prior_delta) in delta.iter_mut().zip(&computed[*influencer]) {
                    *value = value.subtract_scaled(prior_delta.clone(), *scalar);
                }
            }
            computed.push(delta);
        }

        Ok(self
            .order
            .iter()
            .copied()
            .zip(self.supports.iter().cloned())
            .zip(computed)
            .map(|((source_index, support), deltas)| (source_index, support, deltas))
            .collect())
    }
}

pub trait DeltaValue: Clone {
    fn subtract_scaled(&self, other: Self, scalar: f64) -> Self;
}

fn axis_points(locations: &[Vec<f64>], axis_count: usize) -> Vec<Vec<f64>> {
    let mut points = vec![Vec::new(); axis_count];
    for location in locations {
        let non_zero: Vec<_> = location
            .iter()
            .enumerate()
            .filter(|(_, value)| !approximately_zero(**value))
            .collect();
        if non_zero.len() == 1 {
            points[non_zero[0].0].push(*non_zero[0].1);
        }
    }
    for points_for_axis in &mut points {
        points_for_axis.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        points_for_axis.dedup_by(|left, right| approximately_equal(*left, *right));
    }
    points
}

fn location_sort_key(
    location: &[f64],
    axis_points: &[Vec<f64>],
) -> (usize, i16, Vec<usize>, Vec<i8>, Vec<OrderedF64>) {
    let non_zero_axes: Vec<usize> = location
        .iter()
        .enumerate()
        .filter_map(|(axis, value)| (!approximately_zero(*value)).then_some(axis))
        .collect();
    let on_axis_count = location
        .iter()
        .enumerate()
        .filter(|(axis, value)| {
            axis_points[*axis]
                .iter()
                .any(|point| approximately_equal(*point, **value))
        })
        .count() as i16;
    let signs = non_zero_axes
        .iter()
        .map(|axis| location[*axis].signum() as i8)
        .collect();
    let absolutes = non_zero_axes
        .iter()
        .map(|axis| OrderedF64(location[*axis].abs()))
        .collect();
    (
        non_zero_axes.len(),
        -on_axis_count,
        non_zero_axes,
        signs,
        absolutes,
    )
}

fn regions_for(locations: &[Vec<f64>], axis_count: usize) -> Vec<Support> {
    let mut minimum = vec![0.0_f64; axis_count];
    let mut maximum = vec![0.0_f64; axis_count];
    for location in locations {
        for (axis, value) in location.iter().enumerate() {
            minimum[axis] = minimum[axis].min(*value);
            maximum[axis] = maximum[axis].max(*value);
        }
    }
    locations
        .iter()
        .map(|location| Support {
            tents: location
                .iter()
                .enumerate()
                .map(|(axis, peak)| Tent::from_peak(*peak, minimum[axis], maximum[axis]))
                .collect(),
        })
        .collect()
}

fn master_supports(regions: &[Support]) -> Vec<Support> {
    let mut supports = Vec::with_capacity(regions.len());
    for region in regions {
        let mut current = region.clone();
        for previous in &supports {
            if active_axes(&current) != active_axes(previous) || !overlaps(&current, previous) {
                continue;
            }
            let mut best_ratio = -1.0;
            let mut replacements = Vec::new();
            for (axis, (tent, previous_tent)) in
                current.tents.iter().zip(&previous.tents).enumerate()
            {
                let mut replacement = *tent;
                let ratio = if previous_tent.peak < tent.peak {
                    replacement.minimum = previous_tent.peak;
                    (previous_tent.peak - tent.peak) / (tent.minimum - tent.peak)
                } else if previous_tent.peak > tent.peak {
                    replacement.maximum = previous_tent.peak;
                    (previous_tent.peak - tent.peak) / (tent.maximum - tent.peak)
                } else {
                    continue;
                };
                if ratio > best_ratio + EPSILON {
                    best_ratio = ratio;
                    replacements.clear();
                }
                if approximately_equal(ratio, best_ratio) {
                    replacements.push((axis, replacement));
                }
            }
            for (axis, replacement) in replacements {
                current.tents[axis] = replacement;
            }
        }
        supports.push(current);
    }
    supports
}

fn active_axes(support: &Support) -> Vec<usize> {
    support
        .tents
        .iter()
        .enumerate()
        .filter_map(|(axis, tent)| (!tent.is_zero()).then_some(axis))
        .collect()
}

fn overlaps(current: &Support, previous: &Support) -> bool {
    current
        .tents
        .iter()
        .zip(&previous.tents)
        .all(|(tent, old)| {
            approximately_equal(old.peak, tent.peak)
                || (tent.minimum < old.peak && old.peak < tent.maximum)
        })
}

fn delta_weights(locations: &[Vec<f64>], supports: &[Support]) -> Vec<Vec<(usize, f64)>> {
    locations
        .iter()
        .enumerate()
        .map(|(index, location)| {
            supports[..index]
                .iter()
                .enumerate()
                .filter_map(|(previous, support)| {
                    let scalar = support.scalar_at(location);
                    (!approximately_zero(scalar)).then_some((previous, scalar))
                })
                .collect()
        })
        .collect()
}

fn has_duplicate_locations(locations: &[Vec<f64>]) -> bool {
    locations.iter().enumerate().any(|(index, location)| {
        locations[..index].iter().any(|other| {
            location
                .iter()
                .zip(other)
                .all(|(left, right)| approximately_equal(*left, *right))
        })
    })
}

fn approximately_zero(value: f64) -> bool {
    approximately_equal(value, 0.0)
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= EPSILON
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OrderedF64(f64);

impl Eq for OrderedF64 {}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Scalar(f64);

    impl DeltaValue for Scalar {
        fn subtract_scaled(&self, other: Self, scalar: f64) -> Self {
            Self(self.0 - other.0 * scalar)
        }
    }

    #[test]
    fn produces_expected_single_axis_deltas() {
        let model = VariationModel::new(vec![vec![0.0], vec![1.0]]).unwrap();
        let deltas = model
            .deltas(&[vec![Scalar(400.0)], vec![Scalar(700.0)]])
            .unwrap();
        assert!(deltas[0].1.is_default());
        assert_eq!(deltas[1].2, vec![Scalar(300.0)]);
        assert_eq!(
            deltas[1].1.tents[0],
            Tent {
                minimum: 0.0,
                peak: 1.0,
                maximum: 1.0
            }
        );
    }

    #[test]
    fn resolves_intermediate_master_deltas() {
        let model = VariationModel::new(vec![vec![0.0], vec![0.5], vec![1.0]]).unwrap();
        let deltas = model
            .deltas(&[vec![Scalar(0.0)], vec![Scalar(100.0)], vec![Scalar(300.0)]])
            .unwrap();
        assert_eq!(deltas[1].2, vec![Scalar(100.0)]);
        assert_eq!(deltas[2].2, vec![Scalar(300.0)]);
    }
}
