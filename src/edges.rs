use core::panic;

use crate::CORRIDOR_WIDTH;

use crate::types::*;

fn edge_rooms<const N: usize>(r1: &Room<N>, r2: &Room<N>, axis: Axis) -> (usize, PointN<N>, PointN<N>, Axis) {
    let mut left_mid = PointN([0; N]);
    let mut right_mid = PointN([0; N]);

    for i in 0..N {
        left_mid[i] = r1.2[i] - ((r1.2[i] - r1.1[i]) / 2);
        right_mid[i] = r2.1[i] + ((r2.2[i] - r2.1[i]) / 2);
    }

    left_mid[axis] = r1.2[axis];
    right_mid[axis] = r2.1[axis];

    return (r1.0, left_mid, right_mid, axis)
}

fn generate_edges<const N: usize>(rooms: (&[&Room<N>], &[&Room<N>]), axis: Axis, split_pos: PointN<N>, edges: &mut Vec<(usize, PointN<N>, PointN<N>, Axis)>) -> () {
    fn get_point_dist<const N: usize>(r: &&Room<N>, p: PointN<N>) -> (i64, i64) {
        let mut d1 = 0;
        let mut d2 = 0;

        //Sum distance in every dimension
        for axis in 0..N {
            d1 += (p[axis] - r.1[axis]).pow(2);
            d2 += (p[axis] - r.2[axis]).pow(2);
        }

        return (d1, d2)
    }

    if rooms.0.is_empty() || rooms.1.is_empty() {
        return
    }

    let mut left_closest = (rooms.0[0], i64::MAX);
    rooms.0.iter().for_each(|r| {
        let d = get_point_dist(r, split_pos);
        if d.0 < left_closest.1 || d.1 < left_closest.1 {
            left_closest = (r, d.0.min(d.1));           
        }
    });

    let mut right_closest = (rooms.1[0], i64::MAX);
    rooms.1.iter().for_each(|r| {
        let d = get_point_dist(r, split_pos);
        if d.0 < right_closest.1 || d.1 < right_closest.1 {
            right_closest = (r, d.0.min(d.1));           
        }
    });

    edges.push(edge_rooms(left_closest.0, right_closest.0, axis));  

}

pub fn orthogonal_paths<const N: usize>(edges: Vec<(usize, PointN<N>, PointN<N>, Axis)>, map: Vec<Vec<(usize, PointN<N>, PointN<N>)>>) -> Vec<(usize, PointN<N>, PointN<N>, Axis)> {
    assert!(N > 0, "BSP dimension must be greater than zero");
    let mut new_edges = Vec::new();

    for e in edges.iter() {
        let start_pos = e.1;
        let target = e.2;
        //let mut delta: i64 = 0;

        //Select starting tile by scanning tile map for a tile that matches the index of the edge
        let starting_tile = map[0].iter().filter(|t| t.0 == e.0).next().unwrap();
        let mut prev_seg = (e.0, e.1, e.1, e.3);
        
        println!("Starting tile: {:#?}", starting_tile);
        let mut current_tile = starting_tile;

        //Make sure that the starting position is correct for the math
        for axis in 0..N {
            if target[axis] > starting_tile.2[axis] {
                prev_seg.1 = prev_seg.2;
                prev_seg.2[axis] = current_tile.2[axis];
            } else if target[axis] < starting_tile.1[axis] {
                prev_seg.1 = prev_seg.2;
                prev_seg.2[axis] = current_tile.1[axis];
            } else {
                //This coordinate doesn't need to be snapped at all!
                prev_seg.1 = prev_seg.2;
                prev_seg.2[axis] = start_pos[axis];
            }
            new_edges.push(prev_seg)
        }

        while prev_seg.2 != target {
            for axis in 0..N {
                if start_pos[axis] < target[axis] {
                    if prev_seg.2[axis] < target[axis] {
                        if current_tile.2[axis] >= target[axis] {
                            //Target is within this tile, so snap to it
                            prev_seg.1 = prev_seg.2;
                            prev_seg.2[axis] = target[axis];

                            //Push
                            new_edges.push(prev_seg);
                        } else {
                            let tile = map[axis].iter()
                                    .filter(|t| t.1[axis] == current_tile.2[axis] && t.0 != current_tile.0)
                                    //The crossing point must lie inside the candidate tile on every non-moving axis
                                    .filter(|t| (0..N).all(|i| i == axis || (prev_seg.2[i] >= t.1[i] && prev_seg.2[i] <= t.2[i])))
                                    .min_by_key(|t| {
                                        if target[axis] > t.2[axis] {
                                            target[axis] - t.2[axis]
                                        } else {
                                            0
                                        }
                                    });
                            let tile = match tile {
                                    Some(v) => v,
                                    None => { 
                                        print!(
                                            "No room found with bounds that match queried bounds! Critical error in pathing function! Edge: {:?} | Axis: {} | Split Point: {}",
                                            current_tile.2, e.3, current_tile.2[axis]
                                        );
                                        panic!();
                                    }
                                };

                            //Change prev_seg so that it stretches the correct span
                            prev_seg.1 = prev_seg.2;
                            prev_seg.2[axis] = current_tile.2[axis];

                            new_edges.push(prev_seg);
                            
                            //Move bounds
                            current_tile = tile;
                        } 
                    }
                } else {
                    if prev_seg.2[axis] > target[axis] {
                        if current_tile.1[axis] <= target[axis] {
                            prev_seg.1 = prev_seg.2;
                            prev_seg.2[axis] = target[axis];
                            new_edges.push(prev_seg);
                        } else {
                            let tile = map[axis].iter()
                                    .filter(|t| t.2[axis] == current_tile.1[axis] && t.0 != current_tile.0)
                                    //The crossing point must lie inside the candidate tile on every non-moving axis
                                    .filter(|t| (0..N).all(|i| i == axis || (prev_seg.2[i] >= t.1[i] && prev_seg.2[i] <= t.2[i])))
                                    .min_by_key(|t| {
                                        if target[axis] < t.1[axis] {
                                            t.1[axis] - target[axis]
                                        } else {
                                            0
                                        }
                                    });
                            let tile = match tile {
                                    Some(v) => v,
                                    None => { 
                                        print!(
                                            "No room found with bounds that match queried bounds! Critical error in pathing function! | Edge: {:?} | Axis: {} | Split Point: {}",
                                            current_tile.1, e.3, current_tile.1[axis]
                                        );
                                        panic!();
                                    }
                                };

                            //Change prev_seg so that it stretches the correct span
                            prev_seg.1 = prev_seg.2;
                            prev_seg.2[axis] = current_tile.1[axis];

                            new_edges.push(prev_seg);

                            //Update current tile
                            current_tile = tile;
                        }
                    }
                }
            }
        }
    }
    return new_edges
}

pub fn create_corridors<const N: usize>(edges: Vec<(usize, PointN<N>, PointN<N>, Axis)>) -> Vec<(PointN<N>, PointN<N>)> {
    let mut boxes = Vec::<(PointN<N>, PointN<N>)>::new();
    for e in edges {
        let mut c1 = PointN([0; N]);
        let mut c2 = PointN([0; N]);

        for axis in 0..N {
            c1[axis] = e.1[axis].min(e.2[axis]) - CORRIDOR_WIDTH;
            c2[axis] = e.1[axis].max(e.2[axis]) + CORRIDOR_WIDTH;
        }

        boxes.push((c1, c2)); 
    }
    boxes
}

pub fn edge_dfs<'a, const N: usize>(root: &'a BSPNode<Tile<N>>, divisions: u32, edges: &mut Vec<(usize, PointN<N>, PointN<N>, Axis)>) -> Vec<&'a Room<N>> {
    if root.value.split_count < divisions {
        let mut left_rooms = edge_dfs(root.left.as_deref().unwrap(), divisions, edges);
        let right_rooms = edge_dfs(root.right.as_deref().unwrap(), divisions, edges);

        generate_edges(
            (&left_rooms, &right_rooms),
            root.split_d,
            root.left.as_deref().unwrap().value.rc,
            edges,
        );

        left_rooms.extend(right_rooms);
        left_rooms
    } else {
        let mut rooms = Vec::new();
        if let Some(room) = &root.value.room {
            rooms.push(room);
        }
        return rooms
    }
}
