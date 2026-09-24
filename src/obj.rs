use crate::types::*;
// use crate::edges::*;

pub fn create_obj() -> String {
    return String::from("#OBJ file exported by grch-procedural-gen\n o grchDungeon\n")
}

//OBJ is a 3D format. For N > 3 this exports a projection using the first
//three coordinates; for N < 3 the missing coordinates are set to zero.
pub fn add_obj_box<const N: usize>(c1: PointN<N>, c2: PointN<N>, obj_data: &mut String, v: usize) -> usize {
    let get_coord = |p: PointN<N>, axis: usize| -> i64 {
        if axis < N {
            p[axis]
        } else {
            0
        }
    };

    let c1x = get_coord(c1, 0);
    let c1y = get_coord(c1, 1);
    let c1z = get_coord(c1, 2);
    let c2x = get_coord(c2, 0);
    let c2y = get_coord(c2, 1);
    let c2z = get_coord(c2, 2);

    //Add vertices
    obj_data.push_str(&format!("v {} {} {}\n", c1x, c1y, c1z)); //1
    obj_data.push_str(&format!("v {} {} {}\n", c2x, c1y, c1z)); //2
    obj_data.push_str(&format!("v {} {} {}\n", c2x, c2y, c1z)); //3 
    obj_data.push_str(&format!("v {} {} {}\n", c2x, c2y, c2z)); //4
    obj_data.push_str(&format!("v {} {} {}\n", c1x, c2y, c2z)); //5
    obj_data.push_str(&format!("v {} {} {}\n", c2x, c1y, c2z)); //6
    obj_data.push_str(&format!("v {} {} {}\n", c1x, c2y, c1z)); //7
    obj_data.push_str(&format!("v {} {} {}\n", c1x, c1y, c2z));

    let o = v;

    //Add faces
    obj_data.push_str(&format!("f {} {} {}\n", o+1, o+7, o+5));
    obj_data.push_str(&format!("f {} {} {}\n", o+1, o+5, o+8)); // Left g
    obj_data.push_str(&format!("f {} {} {}\n", o+1, o+7, o+3));
    obj_data.push_str(&format!("f {} {} {}\n", o+1, o+3, o+2)); // Front g
    obj_data.push_str(&format!("f {} {} {}\n", o+2, o+3, o+4));
    obj_data.push_str(&format!("f {} {} {}\n", o+2, o+4, o+6)); // Right g
    obj_data.push_str(&format!("f {} {} {}\n", o+1, o+8, o+6));
    obj_data.push_str(&format!("f {} {} {} \n", o+1, o+6, o+2));  // Bottom g
    obj_data.push_str(&format!("f {} {} {}\n", o+7, o+5, o+4));
    obj_data.push_str(&format!("f {} {} {}\n", o+7, o+4, o+3)); // Top g
    obj_data.push_str(&format!("f {} {} {}\n", o+8, o+5, o+4));
    obj_data.push_str(&format!("f {} {} {}\n", o+8, o+4, o+6)); // Back gG

    return o+8;
}
