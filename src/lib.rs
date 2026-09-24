use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicUsize;
use rand::{self, RngExt};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::fs::File;
use std::io::Write;

pub mod types;
use crate::types::*;
pub mod edges;
use crate::edges::*;
pub mod obj;
use crate::obj::*;

pub const SEED: u64 = 184138956713986453;
pub static INDEX: AtomicUsize = AtomicUsize::new(0);

pub const PLACE_DIST: i64 = 20;
pub const CORRIDOR_OFFSET: PointN<3> = PointN([10, 4, 2]);
pub const CORRIDOR_WIDTH: i64 = 20;

pub const ABS_DIST_X: i64 = 0;
pub const ABS_DIST_Y: i64 = 0;
pub const ABS_DIST_Z: i64 = 0;
pub const ROOM_RANDOM_VARIABILITY: f64 = 0.05;
pub const ROOM_SCALE_FACTOR: f64 = 0.08;

//Chance of a room being in any given tile
pub const ROOM_CHANCE: f64 = 2.0 / 3.0;

//3D convenience default. initbt itself accepts PointN<N> for any N > 0.
pub const SIZE: PointN<3> = PointN([2048, 2048, 2048]);

fn split_dfs<const N: usize>(root: &mut BSPNode<Tile<N>>, depth: u32, rng: &mut StdRng) {
    if root.value.split_count < depth {
        root.split(rng);
        split_dfs(root.right.as_mut().unwrap(), depth, rng);
        split_dfs(root.left.as_mut().unwrap(), depth, rng);
    } else {
        return
    }
}

fn construct_room<const N: usize>(tile: Tile<N>, rng: &mut StdRng, obj_data: &mut String, v: &mut usize) -> Option<Room<N>> {
    if tile.traversible == false {
        return None
    }

    let mut lc = tile.lc;
    let mut rc = tile.rc;

    for axis in 0..N {
        let dist_from_edge: i64 = (tile.get_dimension(axis) as f64 * rng.random_range(0.0..ROOM_RANDOM_VARIABILITY)) as i64;

        lc[axis] = ((tile.lc[axis] as f64 + tile.get_dimension(axis) as f64 * ROOM_SCALE_FACTOR) as i64) + dist_from_edge;
        rc[axis] = ((tile.rc[axis] as f64 - tile.get_dimension(axis) as f64 * ROOM_SCALE_FACTOR) as i64) - dist_from_edge;
    }

    let room = Room(
        tile.index,
        lc,
        rc,
        true );

    *v = add_obj_box(room.1, room.2, obj_data, *v);
    return Some(room);

}

fn build_dfs<const N: usize>(root: &mut BSPNode<Tile<N>>, tvec: &mut Vec<Tile<N>>, rng: &mut StdRng, obj_data: &mut String, v: &mut usize) -> () {
    if root.right == None {
            let r= construct_room(root.value, rng, obj_data, v);

            //If a room was constructed, insert it into the tile in the tree node
            if r.is_some() {
                root.value.room = r;
            }
            
            //Push tile to tvec so it can be mapped
            tvec.push(root.value);
    } else {
        build_dfs(root.right.as_deref_mut().unwrap(), tvec, rng, obj_data, v);
        build_dfs(root.left.as_deref_mut().unwrap(), tvec, rng, obj_data, v);       
    }
}

pub fn initbt<const N: usize>(size: PointN<N>, divisions: u32) -> () {
    assert!(N > 0, "BSP dimension must be greater than zero");
    let mut rng = StdRng::seed_from_u64(SEED);

    let mut root = BSPNode{
        value: Tile{index: INDEX.fetch_add(1, Relaxed), lc: PointN([0; N]), rc: size, traversible: false, split_count: 0, room: None},
        right: None,
        left: None,
        split_d: rng.random_range(0..N)
    };

    let mut obj_data = create_obj();
    let mut v: usize = 0;
    let mut tvec = Vec::<Tile<N>>::new();

    split_dfs(&mut root, divisions, &mut rng);
    build_dfs(&mut root, &mut tvec, &mut rng, &mut obj_data, &mut v);
    eprintln!("Structure construction finished, now drawing map");
    
    //Initialize map
    let mut map = Vec::<Vec<(usize, PointN<N>, PointN<N>)>>::new();

    //Map of vectors of the form:
    // (index: u64, left_corner: PointN, right_corner: PointN) 
    for axis in 0..N {
        tvec.sort_by(|t1, t2| t1.rc[axis].cmp(&t2.rc[axis]));
        map.push(tvec.iter().map(|t| (t.index, t.lc, t.rc)).collect());
    }

    //Initialize vector used to store the edges generated in the edge_dfs call chain
    let mut edges = Vec::<(usize, PointN<N>, PointN<N>, Axis)>::new();
  
    edge_dfs(&root, divisions, &mut edges);
    eprintln!("Map collection finished, now running orthogonal pathing algorithm");

    //Call functions to turn the edges into orthogonal paths and then turn those into corridors
    let orthogonal_edges = orthogonal_paths(edges, map);
    let corridors = create_corridors(orthogonal_edges);

    //Looks a little weird, v is just an index used inside the box function
    for cor in corridors {
        //v here is a counter that is used for the OBJ export
        v = add_obj_box(cor.0, cor.1, &mut obj_data, v);
    }

    let mut file = File::create("grch_export.obj").expect("Failed to create file");
    file.write_all(obj_data.as_bytes()).expect("Failed to write to file");
    println!("OBJ file exported");

}
