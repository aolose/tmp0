use std::borrow::Cow;
use std::option::Option;
use std::collections::{HashMap};
use std::{thread};
use std::fs::{read_dir, read_to_string};
use std::ops::Deref;
use std::path::PathBuf;
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
struct Cfg {
    assets: String,
    version: String,
    tooltips: String,
    unpack_dir: String,
    english: String,
    spells: Vec<String>,
    flags: Vec<String>,
    icons: Vec<String>,
    dds: Vec<String>,
}

fn load_cfg() -> Cfg {
    let content = read_to_string(CFG_FILE).expect("open file error");
    match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => panic!("parse cfg fail: {e:?}")
    }
}

fn tokenizer(str: &str) -> Vec<&str> {
    let mut results = vec![];
    let re = &Regex::new(r"([^a-zA-Z0-9\-]+)|([a-zA-Z0-9\-]+)").unwrap();
    for c in re.find_iter(str) {
        results.push(c.as_str());
    }
    results
}

fn hash(str: &str) -> String {
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
#[derive(Clone)]
struct Stats {
    id: String,
    stats_type: String,
    weight: u8,
    data: Box<HashMap<String, String>>,
}

pub struct ParseResult {
    spells: Vec<String>,
    keys: String,
    types: String,
    icons: String,
    dic: String,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            id: String::new(),
            stats_type: String::new(),
            weight: 0,
            data: Box::new(HashMap::new()),
        }
    }
}

fn n2s(n: usize) -> String {
    let mut a = n;
    let base: [u8; 152] = [3, 4, 5, 6, 7, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 127,
        128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169,
        170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190,
        191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211,
        212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232,
        233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253,
        254, 255];

    let ba = 115;
    let bb = 33;
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
    ru.iter().map(|&c| c as char).collect::<String>()
}

fn get_spell_proto(cur: usize, list: &Vec<Stats>) -> usize {
    let spell = &list[cur];
    if let Some(id) = spell.data.get("Using") {
        let len = list.len();
        let mut last = 0;
        let max = if spell.id == id.to_string() {
            spell.weight - 1
        } else {
            spell.weight - 0
        };
        for x in cur + 1..len {
            let s = &list[x];
            if s.id == *id {
                if s.weight > max {
                    continue;
                } else if s.weight == max {
                    last = x;
                    break;
                }
                if last != cur {
                    if list[last].weight > s.weight {
                        continue;
                    }
                }
                last = x;
            }
        }
        last
    } else {
        cur
    }
}

pub fn parse_data() -> ParseResult {
    let rs = load_spell();
    let mut pr = ParseResult {
        spells: vec![],
        keys: "".to_string(),
        types: "".to_string(),
        icons: "".to_string(),
        dic: "".to_string(),
    };
    let spells = rs.0;
    pr.types = rs.1.join(",");
    let mut spells_map: HashMap<String, Vec<usize>> = HashMap::new();
    let len = spells.len() - 1;

    for a in 0..len {
        let key = (&spells[a].id).to_string();
        if let Some(u) = spells_map.get_mut(&key) {
            u.push(a)
        } else {
            spells_map.insert(key, vec![a]);
        }
    }

    for (_, v) in spells_map.iter_mut() {
        v.sort_by(|a, b| {
            spells[*a].weight.cmp(&spells[*b].weight)
        })
    };

    let get_current_attr = |key: String, attr: &str| -> (Option<String>, Option<String>) {
        let sp = spells_map.get(&key).unwrap();
        let mut p = None;
        for a in sp {
            let data = &spells[*a].data;
            if let Some(&ref v) = data.get(&attr.to_string()) {
                return (Some(v.to_string()), None);
            } else if p == None {
                if let Some(&ref v) = data.get("Using") {
                    if *v != key {
                        p = Some(v.to_string());
                    }
                }
            }
        }
        (None, p)
    };
    let get_attr = |key: String, attr: &str| -> Option<String>{
        let mut key = key;
        loop {
            let (r, k) = get_current_attr(key, attr);
            if let Some(rv) = r {
                return Some(rv);
            } else if let Some(p) = k {
                key = p;
            } else {
                break
            }
        }
        None
    };
    let mut spell_ids = spells_map.keys().map(|a| a).collect::<Vec<&String>>();
    spell_ids.sort_by(|&a, &b| {
        let nm = |a: &String| {
            let s = &spells[spells_map.get(a).unwrap()[0]];
            let id = &s.id;
            if "InterruptData" != &s.stats_type {
                replace_str(id, "^[a-zA-Z]", "").to_string()
            } else {
                id.to_string()
            }
        };
        let lv = |a| {
            if let Some(lv) = get_attr(a, "Level") {
                let i: u8 = lv.parse().unwrap();
                if i == 0 {
                    98
                } else {
                    i
                }
            } else {
                99
            }
        };
        let la: u8 = lv(a.to_string());
        let lb: u8 = lv(b.to_string());
        if la != lb {
            la.cmp(&lb)
        } else {
            nm(&a).cmp(&nm(&b))
        }
    });
    let mut idx_map = HashMap::new();
    for i in 0..spell_ids.len() {
        idx_map.insert(spell_ids[i], i);
    }
    let mut hide = vec![];
    let mut show = vec![];

    let mut push = |ns: &Vec<usize>, i| {
        let f = ns[i];
        let len = ns.len();
        let spell: &Stats = &spells[f];
        if let Some(us) = get_attr(spell.id.to_string(), "Using") {
            if spell.id == us {
                if i + 1 < len {
                    if i == 0 {
                        show.push((f, ns[i + 1]));
                    } else {
                        hide.push((f, ns[i + 1]));
                    }
                } else {
                    if i == 0 {
                        show.push((f, f));
                    } else {
                        hide.push((f, f));
                    }
                }
            } else {
                let n = spells_map.get(&us).unwrap()[0];
                if i == 0 {
                    show.push((f, n));
                } else {
                    hide.push((f, n));
                }
            }
        } else {
            if i == 0 {
                show.push((f, f));
            } else {
                hide.push((f, f));
            }
        }
    };

    for s in spell_ids {
        let ns = &spells_map.get(s).unwrap();
        for i in 0..ns.len() {
            push(ns, i)
        }
    }


    let mut keys = vec![];
    let mut idx_key = |k: String| -> String {
        for i in 0..keys.len() {
            if keys[i] == k {
                return n2s(i);
            }
        }
        keys.push(k);
        n2s(keys.len() - 1)
    };

    let mut add_spell_str_to_rss = |n: usize| {
        let is_show = n < show.len();
        let idx = if is_show { n } else { n - show.len() };
        let vc = if is_show { &show } else { &hide };
        let (p, c) = vc[idx];
        let spell = &spells[p];
        let mut mp: HashMap<String, String> = spell.data.deref().clone();
        let us = String::from("Using");
        if p == c {
            if let Some(_) = get_attr(spell.id.to_string(), "Using") {
                mp.insert(us, p.to_string());
            }
        } else {
            mp.insert(us, c.to_string());
        }
        if !is_show {
            mp.insert("i".to_string(), idx_map.get(&spell.id).unwrap().to_string());
        }
        let ix = *&spell.weight as usize;
        let a = CFG.flags[ix].to_string();
        mp.insert("mod".to_string(), a);
        let _ = &pr.spells.push(mp.clone().into_keys().fold(String::new(), |a, b| a + idx_key(b).as_str())
            + mp.into_values().fold(String::new(), |a, b| a + "\x00" + b.as_str()).as_str());
    };

    for i in 0..spells.len() - 1 {
        add_spell_str_to_rss(i)
    }

    println!("{}", pr.spells[0]);

    pr
    // let mini_spell = |spells: Vec<&Stats>| -> String {
    //     let mut b = vec![];
    //     for n in spells {
    //         let mut x = String::new();
    //         let m = n.values
    //             .iter()
    //             .map(|vv| vv.join("\x02"))
    //             .collect::<Vec<String>>().join("\x00");
    //         for k in &n.keys {
    //             let mut i = spell_keys.len();
    //             if let Some(key) = spell_keys.iter().position(|r| r == k) {
    //                 i = key
    //             } else {
    //                 spell_keys.push(format!("{}", k));
    //             };
    //             x += &*n2s(i);
    //         }
    //         b.push(format!("{}\x00{}", x, m))
    //     }
    //     format!("\"{}\"", b.join("\x00").replace('"', "\\\""))
    // };
}


#[derive(Deserialize)]
struct ContentList {
    content: Vec<Lang>,
}

#[derive(Deserialize)]
struct Lang {
    #[serde(rename = "@contentuid")]
    contentuid: String,
    #[serde(rename = "$text")]
    value: String,
}
#[derive(Deserialize)]
struct IconNodeList(Vec<crate::IconAttr>);

#[derive(Deserialize)]
struct IconAttr {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Deserialize)]
struct NodeList(Vec<Node>);

#[derive(Deserialize)]
struct Node {
    attribute: Vec<Attribute>,
}
#[derive(Deserialize)]
struct Attribute {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@value")]
    value: Option<String>,
    #[serde(rename = "@handle")]
    handle: Option<String>,
}

fn split_str<'a>(str: &'a str, regex: &str) -> Vec<&'a str> {
    let re = Regex::new(regex).unwrap();
    re.split(str).into_iter().collect::<Vec<&str>>()
}

fn replace_str<'a>(str: &'a str, regex: &str, replace: &str) -> Cow<'a, str> {
    let re = Regex::new(regex).unwrap();
    re.replace(str, replace)
}

fn match_str<'a>(txt: &'a str, regex: &str) -> Option<Vec<&'a str>> {
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
    path: PathBuf,
    weight: u8,
) -> Vec<Stats> {
    let mut list = vec![];
    let mut spell = Stats::new();
    spell.weight = weight;
    let mut x = 0;
    for line in read_to_string(path.clone()).unwrap().lines() {
        let from = |n| { String::from(&line[n..line.len() - 1]) };
        if line.starts_with("new") {
            if spell.id.len() != 0 {
                list.push(spell);
            }
            spell = Stats::new();
            spell.id = from(11);
        } else if line.starts_with("using") {
            let val = from(7);
            spell.data.insert("Using".to_string(), val);
        } else if line.starts_with("type") {
            spell.stats_type = from(6);
        } else if line.starts_with("data") {
            if let Some(v) = match_str(line, r#""([^"]+)" "([^"]+)""#) {
                if v.len() == 2 {
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
        x = x + 1;
    }
    if spell.id.len() != 0 {
        list.push(spell);
    }
    println!("{:?} parsed {} spells", path.file_name().unwrap(), list.len());
    list
}

fn str_u8(str: String) -> u8 {
    let f: f32 = str.parse().unwrap();
    (f * 32f32) as u8
}

fn load_icons() -> HashMap<String, [u8; 3]> {
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

fn load_spell() -> (Vec<Stats>, Vec<String>) {
    let mut list: Vec<Stats> = vec![];
    let mut spell_types: Vec<String> = vec![];
    let mut i = 0u8;
    let mut threads = vec![];
    for a in &CFG.spells {
        for entry in read_dir(format!("{}/{}", &CFG.unpack_dir, a)).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.file_name();
            if let Some(n) = name {
                if let Some(n) = n.to_str() {
                    if n.starts_with("Spell_") || n.starts_with("Passive") {
                        threads.push(thread::spawn(move || {
                            parse_spell(path, i)
                        }));
                    }
                }
            }
        };
        i = i + 1;
    }

    for t in threads {
        let a = t.join().unwrap();
        for i in a {
            if let Some(&ref spell_type) = &i.data.get("SpellType") {
                if !spell_types.contains(spell_type) {
                    spell_types.push(spell_type.to_string());
                }
            }
            list.push(i)
        }
    }

    (list, spell_types)
}

fn load_lang() -> HashMap<String, String> {
    let mut lang_map: HashMap<String, String> = HashMap::new();
    let pt = format!("{}/{}", &CFG.unpack_dir, &CFG.english);
    let xml = read_to_string(pt.clone()).expect(&format!("open failed:{}", pt));
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

fn load_tooltips() -> HashMap<String, [String; 2]> {
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

    #[test]
    fn test_n2s() {
        let a = n2s(7000);
        assert_eq!(a, "Ìöý");
    }
}
