use std::borrow::Cow;
use std::option::Option;
use std::collections::HashMap;
use std::fs;
use std::fs::{read_dir, read_to_string};
use toml;
use serde::Deserialize;
use regex::Regex;
use quick_xml::de;

const CFG_FILE: &str = "./cfg.toml";

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

pub fn load_cfg() -> Cfg {
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

pub struct Spell {
    id: String,
    flag: String,
    data: Box<HashMap<String, String>>,
    proto: Option<Box<Spell>>,
}

impl Spell {
    pub fn new() -> Spell {
        Spell {
            id: String::new(),
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

fn parse_spell(list: &mut Vec<Spell>, slices: Vec<&str>, start: usize,flag:String) {
    if start == slices.len() { return; };
    let mut i = 0;
    let mut spell = Spell::new();
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
                    // todo tooltips[c].Name + '<br>' + tooltips[c].Text;
                    spell.data.insert(key.to_string(), c.to_string());
                } else if ["DisplayName", "Description", "ExtraDescription"].contains(&key) {
                    let d = replace_str(&*c, r";\d+$", "");
                    // todo  e[b] = lang[d] || d;
                    spell.data.insert(key.to_string(), d.to_string());
                }else {
                    let d = if c=="unknown" { "".to_string() }else {  c.replace(";","\x02") };
                    spell.data.insert(key.to_string(), d);
                }
            }
        }
    }
    // todo  if (e.SpellType) types.add(e.SpellType);
    list.push(spell);
    parse_spell(list, slices, start + i,flag);
}

pub fn load_spell() {
    let cfg = load_cfg();
    let mut list: Vec<Spell> = vec![];
    let mut spell_files = vec![];
    for [a, flag] in cfg.spells {
        for entry in read_dir(format!("{}/{}", &cfg.unpack_dir, a)).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.file_name();
            if let Some(n) = name {
                if let Some(n) = n.to_str() {
                    if n.starts_with("Spell_") {
                        spell_files.push((path, flag.clone()));
                    }
                }
            }
        };
    }
    for (path, flag) in spell_files {
        let file = fs::read_to_string(path).unwrap();
        let mut i = -1;
        parse_spell(&mut list, split_str(&file, "\r?\n"), 0,flag);
    }
}

pub fn load_lang() -> HashMap<String, String> {
    let mut lang_map: HashMap<String, String> = HashMap::new();
    let cfg = load_cfg();
    let xml = read_to_string(cfg.unpack_dir + "/" + &cfg.english).expect("open file error");
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

pub fn load_tooltips(lang: Option<&HashMap<String, String>>) -> HashMap<String, String> {
    let mut tooltips_map: HashMap<String, String> = HashMap::new();
    let cfg = load_cfg();
    let xml = read_to_string(cfg.unpack_dir + "/" + &cfg.tooltips).expect("open file error");
    let re = Regex::new("<children>|</children>").unwrap();
    let slices = re.split(&*xml).into_iter().collect::<Vec<&str>>();
    let slice = slices[1];
    let list: NodeList = de::from_str(slice).unwrap();
    list.0.iter().for_each(|node| {
        let mut uuid = "";
        let mut text = "";
        node.attribute.iter().for_each(|b| {
            let Attribute { id, value, handle } = b;
            if id == "UUID" {
                if let Some(a) = value {
                    uuid = a
                }
            } else {
                if text == "" {
                    if let Some(b) = value {
                        text = b
                    }
                }
                if id == "Text" {
                    if let Some(b) = handle {
                        if let Some(l) = lang {
                            if let Some(c) = l.get(b) {
                                text = c;
                                return;
                            }
                        }
                        if text == "" {
                            text = b
                        }
                    }
                }
            }
        });
        tooltips_map.insert(uuid.to_string(), text.to_string());
    });
    tooltips_map
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_match_str() {
        let txt = r#"data "sates" "value""#;
        let matches = match_str(&txt, r#""([^"]+)" "([^"]+)""#);
        assert_eq!("sates", matches.unwrap()[0])
    }
    #[test]
    fn read_tooltips() {
        let lang = load_tooltips(None);
        let val = lang.get("66388a6f-44dd-4c9f-a9e7-910c50e70755");
        assert_eq!(val, Some(&"Additional damage".to_string()));
    }
    #[test]
    fn read_lang() {
        let lang = load_lang();
        let val = lang.get("h000006d4gcefbg4092gbb39gfeb27a3bb0a7");
        assert_eq!(val, Some(&"Sorry, darling, I haven't got time for underlings.".to_string()));
    }
    #[test]
    fn read_cfg_works() {
        let cfg = load_cfg();
        assert_eq!(cfg.assets, "public");
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