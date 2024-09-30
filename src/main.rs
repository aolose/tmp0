use learn0::match_str;

fn main() {
    let txt = r#"data "stest" "value""#;
    let matches = match_str(&txt,r#""([^"]+)" "([^"]+)""#);
    println!("{:?}", matches);
}