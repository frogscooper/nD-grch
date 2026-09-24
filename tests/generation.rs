use std::fs;
use std::sync::{Mutex, MutexGuard};
use rand::rngs::StdRng;
use rand::SeedableRng;
use nD_grch::*;
use nD_grch::edges::*;
use nD_grch::types::*;

//All initbt tests write to the same OBJ path, so keep those tests from racing
//each other when the Rust test runner uses multiple threads.
static GENERATION_LOCK: Mutex<()> = Mutex::new(());

fn generation_lock() -> MutexGuard<'static, ()> {
    GENERATION_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn straight_map<const N: usize>() -> Vec<Vec<(usize, PointN<N>, PointN<N>)>> {
    assert!(N > 0);

    let mut tiles = Vec::new();

    for i in 0..3 {
        let mut lc = [0; N];
        let mut rc = [100; N];

        lc[0] = i * 100;
        rc[0] = (i + 1) * 100;

        tiles.push((i as usize + 1, PointN(lc), PointN(rc)));
    }

    let mut map = Vec::new();
    for axis in 0..N {
        let mut sorted = tiles.clone();
        sorted.sort_by(|t1, t2| t1.2[axis].cmp(&t2.2[axis]));
        map.push(sorted);
    }

    return map
}

fn check_straight_route<const N: usize>() {
    assert!(N > 0);

    let map = straight_map::<N>();

    let mut start = PointN([0; N]);
    let mut target = PointN([0; N]);
    start[0] = 80;
    target[0] = 220;

    let path = orthogonal_paths(
        vec![(1, start, target, 0)],
        map.clone(),
    );

    assert!(!path.is_empty());
    assert_eq!(path.first().unwrap().1, start);
    assert_eq!(path.last().unwrap().2, target);

    for segment in path.iter() {
        let changed_axes = (0..N)
            .filter(|axis| segment.1[*axis] != segment.2[*axis])
            .count();

        assert!(
            changed_axes <= 1,
            "Route contained a non-orthogonal segment: {:?}",
            segment
        );

        assert!(segment.2[0] >= segment.1[0]);

        for axis in 1..N {
            assert_eq!(segment.1[axis], 0);
            assert_eq!(segment.2[axis], 0);
        }
    }

    for segments in path.windows(2) {
        assert_eq!(segments[0].2, segments[1].1);
    }

    //Repeat the same route in the decreasing direction. This exercises the
    //other half of the boundary traversal logic.
    let reverse_path = orthogonal_paths(
        vec![(3, target, start, 0)],
        map,
    );

    assert!(!reverse_path.is_empty());
    assert_eq!(reverse_path.first().unwrap().1, target);
    assert_eq!(reverse_path.last().unwrap().2, start);

    for segment in reverse_path.iter() {
        let changed_axes = (0..N)
            .filter(|axis| segment.1[*axis] != segment.2[*axis])
            .count();

        assert!(
            changed_axes <= 1,
            "Reverse route contained a non-orthogonal segment: {:?}",
            segment
        );

        assert!(segment.2[0] <= segment.1[0]);
    }

    for segments in reverse_path.windows(2) {
        assert_eq!(segments[0].2, segments[1].1);
    }
}

fn check_split<const N: usize>(split_axis: usize) {
    assert!(N > 0);
    assert!(split_axis < N);

    let lc = PointN([0; N]);
    let rc = PointN([1000; N]);

    let mut root = BSPNode {
        value: Tile {
            index: 0,
            lc,
            rc,
            traversible: false,
            split_count: 0,
            room: None,
        },
        left: None,
        right: None,
        split_d: split_axis,
    };

    let mut rng = StdRng::seed_from_u64(SEED);
    root.split(&mut rng);

    let left = root.left.as_ref().expect("Split did not create a left child");
    let right = root.right.as_ref().expect("Split did not create a right child");

    assert_eq!(left.value.lc, root.value.lc);
    assert_eq!(right.value.rc, root.value.rc);
    assert_eq!(left.value.split_count, 1);
    assert_eq!(right.value.split_count, 1);

    assert!(left.value.rc[split_axis] > root.value.lc[split_axis]);
    assert!(left.value.rc[split_axis] < root.value.rc[split_axis]);
    assert_eq!(left.value.rc[split_axis], right.value.lc[split_axis]);

    for axis in 0..N {
        if axis != split_axis {
            assert_eq!(left.value.rc[axis], root.value.rc[axis]);
            assert_eq!(right.value.lc[axis], root.value.lc[axis]);
        }
    }

    if N == 1 {
        assert_eq!(left.split_d, 0);
        assert_eq!(right.split_d, 0);
    } else {
        assert!(left.split_d < N);
        assert!(right.split_d < N);
        assert_ne!(left.split_d, split_axis);
        assert_ne!(right.split_d, split_axis);
    }
}

#[test]
fn generates_obj_successfully() {
    let _lock = generation_lock();
    let path = "grch_export.obj";
    let _ = fs::remove_file(path);

    initbt(PointN::<3>([2048, 2048, 2048]), 6);

    let metadata = fs::metadata(path)
        .expect("OBJ file was not created");

    assert!(
        metadata.len() > 0,
        "OBJ file was created but is empty"
    );
}

#[test]
fn deterministic_generation() {
    let _lock = generation_lock();

    initbt(PointN::<3>([2048, 2048, 2048]), 6);

    let first = fs::read("grch_export.obj")
        .expect("Failed to read first OBJ");

    initbt(PointN::<3>([2048, 2048, 2048]), 6);

    let second = fs::read("grch_export.obj")
        .expect("Failed to read second OBJ");

    assert_eq!(first, second);
}

#[test]
fn obj_contains_geometry() {
    let _lock = generation_lock();

    initbt(PointN::<3>([2048, 2048, 2048]), 6);

    let obj = fs::read_to_string("grch_export.obj")
        .expect("Failed to read generated OBJ");
    
    assert!(
        obj.lines().any(|line| line.starts_with("v ")),
        "OBJ contains no vertices"
    );
    assert!(
        obj.lines().any(|line| line.starts_with("f ")),
        "OBJ contains no faces"
    );
}

#[test]
fn pointn_arithmetic_and_indexing_work_in_higher_dimensions() {
    let a = PointN::<5>([1, 2, 3, 4, 5]);
    let b = PointN::<5>([5, 4, 3, 2, 1]);

    assert_eq!(a + b, PointN::<5>([6, 6, 6, 6, 6]));
    assert_eq!(b - a, PointN::<5>([4, 2, 0, -2, -4]));
    assert_eq!(a.sum(), 15);

    let mut point = PointN::<7>([0; 7]);
    point[5] = 42;

    assert_eq!(point[5], 42);
    assert_eq!(point.0[5], 42);
}

#[test]
fn bsp_split_preserves_partition_in_multiple_dimensions() {
    check_split::<1>(0);
    check_split::<2>(1);
    check_split::<4>(2);
    check_split::<6>(5);
}

#[test]
fn orthogonal_pathing_works_in_multiple_dimensions() {
    check_straight_route::<1>();
    check_straight_route::<2>();
    check_straight_route::<4>();
    check_straight_route::<6>();
}

#[test]
fn next_split_uses_largest_remaining_dimension() {
    let mut root = BSPNode {
        value: Tile {
            index: 0,
            lc: PointN::<4>([0, 0, 0, 0]),
            rc: PointN::<4>([100, 200, 500, 300]),
            traversible: false,
            split_count: 0,
            room: None,
        },
        left: None,
        right: None,
        split_d: 2,
    };

    let mut rng = StdRng::seed_from_u64(SEED);
    root.split(&mut rng);

    //Axis 2 is the current split axis, so axis 3 is the largest remaining axis.
    assert_eq!(root.left.as_ref().unwrap().split_d, 3);
    assert_eq!(root.right.as_ref().unwrap().split_d, 3);
}

#[test]
fn corridor_bounds_expand_all_dimensions() {
    let edge = (
        1,
        PointN::<5>([50, 20, 30, 40, 60]),
        PointN::<5>([10, 20, 70, 5, 60]),
        0,
    );

    let corridors = create_corridors(vec![edge]);

    assert_eq!(corridors.len(), 1);
    assert_eq!(corridors[0].0, PointN::<5>([-10, 0, 10, -15, 40]));
    assert_eq!(corridors[0].1, PointN::<5>([70, 40, 90, 60, 80]));
}

#[test]
fn full_generation_runs_outside_three_dimensions() {
    let _lock = generation_lock();

    initbt(PointN::<1>([2048]), 3);
    initbt(PointN::<2>([2048, 2048]), 3);
    initbt(PointN::<4>([2048, 2048, 2048, 2048]), 3);
    initbt(PointN::<5>([2048, 2048, 2048, 2048, 2048]), 3);

    let metadata = fs::metadata("grch_export.obj")
        .expect("n-dimensional generation did not create an OBJ projection");

    assert!(metadata.len() > 0);
}

#[test]
#[should_panic(expected = "BSP dimension must be greater than zero")]
fn zero_dimensional_generation_is_rejected() {
    initbt(PointN::<0>([]), 1);
}
