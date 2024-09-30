use std::borrow::Cow;
use std::option::Option;
use std::collections::HashMap;
use std::fs;
use std::fs::{read_dir, read_to_string};
use toml;
use serde::Deserialize;
use regex::Regex;
use quick_xml::de;
use lazy_static::lazy_static;
const CFG_FILE: &str = "./cfg.toml";

lazy_static! {
    static ref CFG:Cfg = load_cfg();
    static ref LANG_EN:HashMap<String,String> = load_lang();
    static ref TOOLTIP_MAP: HashMap<String,[String;2]> = load_tooltips();
}

#[derive(Deserialize)]
pub struct Cfg {
    pub assets: String,
    pub version: String,
    pub tooltips: String,
    pub unpack_dir: String,
    pub english: String,
    pub spells: Vec<[String; 2]>,
    pub icons: Vec<String>,
    pub dds: Vec<String>,
}

fn load_cfg() -> Cfg {
    let content = read_to_string(CFG_FILE).expect("open file error");
    match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => panic!("parse cfg fail: {e:?}")
    }
}

pub fn tokenizer(str: &str) -> Vec<&str> {
    let mut results = vec![];
    let re = &Regex::new(r"([^a-zA-Z0-9\-]+)|([a-zA-Z0-9\-]+)").unwrap();
    for c in re.find_iter(str) {
        results.push(c.as_str());
    }
    results
}

pub fn hash(str: &str) -> String {
    let mut a: u32 = 0;
    let mut result = vec![];
    for chr in str.bytes() {
        a = a.overflowing_shl(5).0.overflowing_sub(a).0 + chr as u32;
    }
    loop {
        result.push(std::char::from_digit((a % 36) as u32, 36).unwrap());
        a = a / 36;
        if a == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

pub struct Stats {
    id: String,
    stats_type: String,
    flag: String,
    data: Box<HashMap<String, String>>,
    pub proto: Option<Box<Stats>>,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            id: String::new(),
            stats_type: String::new(),
            flag: String::new(),
            data: Box::new(HashMap::new()),
            proto: None,
        }
    }
}


pub fn n2s(n: usize) -> String {
    let mut a = n;
    let base: [u8; 152] = [3, 4, 5, 6, 7, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 127,
        128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169,
        170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190,
        191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
        212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232,
        233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253,
        254, 255];

    let ba = 111;
    let bb = 38;
    let base_str = &base[..ba];
    let t_str = &base[ba..ba + bb];
    let h_str = &base[ba + bb..];
    let mut ru = vec![];
    ru.push(base_str[a % ba]);
    if a > ba {
        a = a / ba;
        ru.push(t_str[a % bb]);
    }
    if a > bb {
        a = a / bb;
        ru.push(h_str[a]);
    }
    let v = ru.into_iter().rev().collect::<Vec<u8>>();
    String::from_utf8(v).expect("parse error")
}


// pub fn parse_data() {
//     // let cfg = load_cfg("./cfg.toml");
//     // let mut spells: Vec<&str> = vec![];
//     let mut spell_keys: Vec<String> = vec![];
//     // let mut used_icons: Vec<&str> = vec![];
//     // let task: Vec<(String, String)> = vec![];
//
//     let mini_spell = |spells: Vec<&Spell>| -> String {
//         let mut b = vec![];
//         for n in spells {
//             let mut x = String::new();
//             let m = n.values
//                 .iter()
//                 .map(|vv| vv.join("\x02"))
//                 .collect::<Vec<String>>().join("\x00");
//             for k in &n.keys {
//                 let mut i = spell_keys.len();
//                 if let Some(key) = spell_keys.iter().position(|r| r == k) {
//                     i = key
//                 } else {
//                     spell_keys.push(format!("{}", k));
//                 };
//                 x += &*n2s(i);
//             }
//             b.push(format!("{}\x00{}", x, m))
//         }
//         format!("\"{}\"", b.join("\x00").replace('"', "\\\""))
//     };
// }
#[derive(Deserialize)]
pub struct ContentList {
    pub content: Vec<Lang>,
}

#[derive(Deserialize)]
pub struct Lang {
    #[serde(rename = "@contentuid")]
    pub contentuid: String,
    #[serde(rename = "$text")]
    pub value: String,
}
#[derive(Deserialize)]
pub struct IconNodeList(Vec<crate::IconAttr>);

#[derive(Deserialize)]
pub struct IconAttr {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Deserialize)]
pub struct NodeList(Vec<Node>);

#[derive(Deserialize)]
pub struct Node {
    pub attribute: Vec<Attribute>,
}
#[derive(Deserialize)]
pub struct Attribute {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@value")]
    pub value: Option<String>,
    #[serde(rename = "@handle")]
    pub handle: Option<String>,
}

pub fn split_str<'a>(str: &'a str, regex: &str) -> Vec<&'a str> {
    let re = Regex::new(regex).unwrap();
    re.split(str).into_iter().collect::<Vec<&str>>()
}

pub fn replace_str<'a>(str: &'a str, regex: &str, replace: &str) -> Cow<'a, str> {
    let re = Regex::new(regex).unwrap();
    re.replace(str, replace)
}

pub fn match_str<'a>(txt: &'a str, regex: &str) -> Option<Vec<&'a str>> {
    let re = Regex::new(regex).unwrap();
    let mut v = vec![];
    for cap in re.captures_iter(txt) {
        for m in cap.iter().skip(1) {
            if let Some(a) = m {
                v.push(&txt[a.start()..a.end()])
            } else {
                return None;
            }
        }
    }
    if v.len() == 0 {
        None
    } else {
        Some(v)
    }
}

fn parse_spell(
    spell_types: &mut Vec<String>,
    list: &mut Vec<Stats>,
    slices: Vec<&str>,
    start: usize,
    flag: String,
) {
    if start == slices.len() { return; };
    let mut i = 0;
    let mut spell = Stats::new();
    spell.flag = flag.clone();
    for n in slices[start..].iter() {
        if n.starts_with("new entry") {
            if i == 0 {
                i = i + 1;
                spell.id = n.replace("new entry ", "");
                continue;
            } else {
                break;
            }
        }
        i = i + 1;
        if n.starts_with("using") {
            let val = n[6..].replace("\"", "");
            spell.data.insert("Using".to_string(), val);
        } else if n.starts_with("type ") {
            spell.stats_type = n[6..n.len() - 1].to_string()
        } else if n.starts_with("data ") {
            if let Some(v) = match_str(n, r#""([^"]+)" "([^"]+)""#) {
                if v.len() != 2 {
                    continue;
                }
                let key = v[0];
                let value = v[1];
                let c = replace_str(
                    value,
                    r"([a-zA-Z]+\([0-9',.+\-a-zA-Z \/\\()_]*\))",
                    "<b>$1</b>",
                );
                if key == "TooltipUpcastDescription" {
                    let text;
                    let hash = c.to_string();
                    if let Some([name, val]) = &TOOLTIP_MAP.get(&hash) {
                        text = format!("{}<br>{}", name, val);
                    } else { text = hash }
                    spell.data.insert(key.to_string(), text);
                } else if ["DisplayName", "Description", "ExtraDescription"].contains(&key) {
                    let d = replace_str(&*c, r";\d+$", "");
                    spell.data.insert(
                        key.to_string(),
                        if let Some(v) = &LANG_EN.get(&d.to_string()) {
                            v.to_string()
                        } else {
                            d.to_string()
                        },
                    );
                } else {
                    let d = if c == "unknown" { "".to_string() } else { c.replace(";", "\x02") };
                    spell.data.insert(key.to_string(), d);
                }
            }
        }
    }
    if let Some(spell_type) = spell.data.get("SpellType") {
        if spell_types.contains(&spell_type) {
            spell_types.push(spell_type.to_string());
        }
    }
    list.push(spell);
    parse_spell(spell_types, list, slices, start + i, flag);
}

fn str_u8(str: String) -> u8 {
    let f: f32 = str.parse().unwrap();
    (f * 32f32) as u8
}

pub fn load_icons() -> HashMap<String, [u8; 3]> {
    let mut icons_map: HashMap<String, [u8; 3]> = HashMap::new();
    let mut x = 0u8;
    for f in &CFG.icons {
        x = x + 1;
        let str = read_to_string(format!("{}/{}", &CFG.unpack_dir, f)).unwrap();
        for attrs in split_str(str.as_str(), "<node id=\"IconUV\">|</node>")
            .iter().filter(|a| a.contains("<attribute id=\"MapKey")) {
            let attrs: IconNodeList = de::from_str(attrs).unwrap();
            let mut id = String::new();
            let mut u = 0;
            let mut v = 0;

            for a in attrs.0 {
                match a.id.as_str() {
                    "MapKey" => id = a.value.to_string(),
                    "U1" => u = str_u8(a.value),
                    "V1" => v = str_u8(a.value),
                    _ => ()
                }
            }
            icons_map.insert(id, [u, v, x - 1]);
        }
    }
    icons_map
}


pub fn load_spell() {
    let mut list: Vec<Stats> = vec![];
    let mut spell_types: Vec<String> = vec![];
    let mut spell_files = vec![];
    for [a, flag] in &CFG.spells {
        for entry in read_dir(format!("{}/{}", &CFG.unpack_dir, a)).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.file_name();
            if let Some(n) = name {
                if let Some(n) = n.to_str() {
                    if n.starts_with("Spell_") || n.starts_with("Passive") {
                        spell_files.push((path, flag.clone()));
                    }
                }
            }
        };
    }
    for (path, flag) in spell_files {
        let file = fs::read_to_string(path).unwrap();
        parse_spell(&mut spell_types, &mut list, split_str(&file, "\r?\n"), 0, flag);
    }
}

pub fn load_lang() -> HashMap<String, String> {
    let mut lang_map: HashMap<String, String> = HashMap::new();
    let xml = read_to_string(format!("{}/{}", CFG.unpack_dir, CFG.english)).expect("open file error");
    let a: ContentList = de::from_str(&*xml).expect("de error");
    a.content.iter().for_each(|a| {
        let Lang { contentuid: k, value: v } = a;
        lang_map.insert(
            k.to_string(),
            v.to_string(),
        );
    });
    lang_map
}

pub fn load_tooltips() -> HashMap<String, [String; 2]> {
    let mut tooltips_map: HashMap<String, [String; 2]> = HashMap::new();
    let xml = read_to_string(format!("{}/{}", CFG.unpack_dir, CFG.tooltips))
        .expect("open file error");
    let re = Regex::new("<children>|</children>").unwrap();
    let slices = re.split(&*xml).into_iter().collect::<Vec<&str>>();
    let slice = slices[1];
    let list: NodeList = de::from_str(slice).unwrap();
    list.0.iter().for_each(|node| {
        let mut uuid = "";
        let mut text = "";
        let mut name = "";
        node.attribute.iter().for_each(|b| {
            let Attribute { id, value, handle } = b;
            match &id[0..1] {
                "N" => if let Some(v) = value { name = v }
                "T" => if let Some(v) = handle {
                    if let Some(vv) = LANG_EN.get(v) {
                        text = vv
                    } else {
                        text = v
                    }
                },
                "U" => if let Some(v) = value { uuid = v }
                _ => ()
            }
        });
        tooltips_map.insert(uuid.to_string(), [name.to_string(), text.to_string()]);
    });
    tooltips_map
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_icons() {
        let mp = load_icons();
        if let Some(i) = mp.get("statIcons_YeenoghusHunger") {
            assert_eq!(i, &[6, 11, 0]);
        } else {
            assert!(false, "should never reach here")
        };
    }

    #[test]
    fn test_match_str() {
        let txt = r#"data "sates" "value""#;
        let matches = match_str(&txt, r#""([^"]+)" "([^"]+)""#);
        assert_eq!("sates", matches.unwrap()[0])
    }
    #[test]
    fn read_tooltips() {
        let lang = load_tooltips();
        let val = lang.get("66388a6f-44dd-4c9f-a9e7-910c50e70755").unwrap();
        assert_eq!(val[0], "Additional damage");
    }
    #[test]
    fn read_lang() {
        let lang = load_lang();
        let val = lang.get("h000006d4gcefbg4092gbb39gfeb27a3bb0a7");
        assert_eq!(val, Some(&"Sorry, darling, I haven't got time for underlings.".to_string()));
    }
    #[test]
    fn read_cfg_works() {
        assert_eq!(CFG.assets, "public");
    }

    #[test]
    fn split_words() {
        let sp = tokenizer("Aa Ab-c1_2");
        let expect = vec!["Aa", " ", "Ab-c1", "_", "2"];
        assert_eq!(sp, expect);
    }

    #[test]
    fn hash_text() {
        let a = "hello world!";
        assert_eq!(hash(a), "1vfqu3h");
    }
}