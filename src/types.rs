use std::ops::*;
use std::sync::atomic::Ordering;
use rand::RngExt;
use rand::rngs::StdRng;
use crate::INDEX;
use crate::ROOM_CHANCE;

pub type Axis = usize;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct BSPNode<T> {
    pub value: T,
    pub left: Option<Box<BSPNode<T>>>,
    pub right: Option<Box<BSPNode<T>>>,
    pub split_d: Axis
}

impl<const N: usize> BSPNode<Tile<N>> {
    pub fn split(&mut self, rng: &mut StdRng) {
            assert!(N > 0, "BSP dimension must be greater than zero");
            assert!(self.split_d < N, "Split axis out of bounds");

            let rng_factor = rng.random_range(0.3..0.7);

            //Choose the largest dimension other than the one we just split on.
            //For N == 1, the only possible next split is axis 0 again.
            let mut next_split = self.split_d;
            if N > 1 {
                next_split = if self.split_d == 0 { 1 } else { 0 };

                for axis in 0..N {
                    if axis != self.split_d && self.value.get_dimension(axis) > self.value.get_dimension(next_split) {
                        next_split = axis;
                    }
                }
            }

            match self.left.as_ref() {
                Some(_) => (),
                None => {
                    let mut left_rc = self.value.rc;
                    left_rc[self.split_d] = self.value.rc[self.split_d]
                        - (self.value.get_dimension(self.split_d) as f64 * rng_factor) as i64;

                    self.left = Some(Box::from(BSPNode{
                        value: Tile {
                            index: INDEX.fetch_add(1, Ordering::Relaxed),
                            lc: self.value.lc,
                            rc: left_rc,
                            traversible: rng.random_bool(ROOM_CHANCE),
                            split_count: (self.value.split_count + 1),
                            room: None
                        },
                        left:  None,
                        right: None,
                        split_d: next_split
                    }));

                    let mut right_lc = self.value.lc;
                    right_lc[self.split_d] = self.left.as_ref().unwrap().value.rc[self.split_d];
                    
                    self.right = Some(Box::from(BSPNode{
                        value: Tile { 
                            index: INDEX.fetch_add(1, Ordering::Relaxed),
                            lc: right_lc,
                            rc: self.value.rc,
                            traversible: rng.random_bool(ROOM_CHANCE),
                            split_count: (self.value.split_count + 1),
                            room: None
                        },
                        left:  None,
                        right: None,
                        split_d: next_split
                    }));
                }
            }
        }
    }

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub struct Tile<const N: usize> {
    pub index: usize,
    pub lc: PointN<N>,
    pub rc: PointN<N>,
    pub traversible: bool,
    pub split_count: u32,
    pub room: Option<Room<N>>
}

impl<const N: usize> Tile<N> {
    pub fn get_dimension(&self, axis: Axis) -> i64 {
        return self.rc[axis] - self.lc[axis]
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub struct PointN<const N: usize>(pub [i64; N]);

impl<const N: usize> PointN<N> {
    pub fn sum(&self) -> i64 {
        let mut sum = 0;
        for n in &self.0 {
            sum += n;
        }
        return sum
    }
}

impl<const N: usize> std::ops::Index<usize> for PointN<N> {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
       return &self.0[index];
    }
}

impl<const N: usize> std::ops::IndexMut<usize> for PointN<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return &mut self.0[index]
    }
}

impl<const N: usize> Sub<PointN<N>> for PointN<N> {
    type Output = PointN<N>;

    fn sub(self, rhs: Self) -> Self {
        let mut new = [0; N];
        for i in 0..N {
            new[i] = self[i] - rhs[i]
        }   
        return Self(new)
    }
}

impl<const N: usize> Add<PointN<N>> for PointN<N> {
    type Output = PointN<N>;

    fn add(self, rhs: Self) -> Self {
        let mut new = [0; N];
        for i in 0..N {
            new[i] = self[i] + rhs[i]
        }   
        return Self(new)
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub struct Room<const N: usize>(pub usize, pub PointN<N>, pub PointN<N>, pub bool);
