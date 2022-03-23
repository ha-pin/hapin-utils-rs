pub mod transform_to_hapin;
mod utils;
// use transform_to_hapin::transform_to_hapin;

fn main() {
    let c = "ٴا";
    println!("{:?} {:?}", utils::get_char(c, 0), utils::get_char(c, 0) as u32)
}