use p_01_hard_regex::SimpleRegex;

fn main() {
    let r: SimpleRegex = ".*".parse::<SimpleRegex>().unwrap();
    println!("Regex works: {}", r.check("Hello world!"));
}
