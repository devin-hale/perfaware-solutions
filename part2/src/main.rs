use std::{collections::HashMap, fs, iter::Peekable, str::Chars};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CoordPair {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl CoordPair {
    fn rand_in_square(x: f64, y: f64) -> CoordPair {
        let mut rng = rand::rng();
        let delta: f64 = rng.random_range(20.0..50.00);
        let x_min = (x - delta).max(-180.0);
        let x_max = (x + delta).min(180.0);
        let y_min = (y - delta).max(-90.0);
        let y_max = (y + delta).min(90.0);

        CoordPair {
            x0: rng.random_range(x_min..=x_max),
            y0: rng.random_range(y_min..=y_max),
            x1: rng.random_range(x_min..=x_max),
            y1: rng.random_range(y_min..=y_max),
        }
    }

    fn rand_set_clustered(len: usize) -> Vec<CoordPair> {
        let mut rng = rand::rng();
        let cluster_count: usize = 50;
        let cluster_size: usize = len / cluster_count;

        let mut pairs = vec![];
        for _ in 0..cluster_count {
            let x = rng.random_range(-180.0..=180.0);
            let y = rng.random_range(-90.0..90.0);
            for _ in 0..cluster_size {
                pairs.push(CoordPair::rand_in_square(x, y));
            }
        }

        pairs
    }
}

fn square(a: f64) -> f64 {
    let result = a * a;
    result
}

fn rad_from_deg(deg: f64) -> f64 {
    let result = 0.01745329251994329577 * deg;
    result
}

const EARTH_RADIUS: f64 = 6372.8;

fn ref_haversine(x0: f64, y0: f64, x1: f64, y1: f64, earth_radius: f64) -> f64 {
    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let dlat = rad_from_deg(lat2 - lat1);
    let dlon = rad_from_deg(lon2 - lon1);

    let lat1 = rad_from_deg(lat1);
    let lat2 = rad_from_deg(lat2);

    let a = square((dlat / 2.0).sin()) + lat1.cos() * lat2.cos() * square((dlon / 2.0).sin());
    let c = 2.0 * a.sqrt().asin();

    let result = earth_radius * c;

    result
}

fn generate_haversine_io() {
    let set_len: usize = 10_000_000;
    let pairs = CoordPair::rand_set_clustered(set_len);

    let haversines: Vec<f64> = pairs
        .iter()
        .map(|p| ref_haversine(p.x0, p.y0, p.x1, p.y1, EARTH_RADIUS))
        .collect();
    let avg_sum = haversines.iter().sum::<f64>() / set_len as f64;
    println!("Pair Count: {set_len}");
    println!("Average Sum: {avg_sum}");

    let pair_json = serde_json::json!({
        "pairs": serde_json::to_value(pairs).unwrap()
    });
    let pair_json = pair_json.to_string();
    let calcs: Vec<String> = haversines.iter().map(|h| (*h).to_string()).collect();
    let calc_str = calcs.join("\n");

    fs::write("./haversine_pairs.json", pair_json).unwrap();
    fs::write("./haversine_calcs.f64", calc_str).unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    String(String),
    Number(f64),
    Colon,
    Comma,
    True,
    False,
    Null,
}

fn tokenize_number(c: char, iter: &mut Peekable<Chars<'_>>) -> Result<f64, String> {
    let mut prd = false;
    let mut s = String::from(c);

    while let Some(p) = iter.peek() {
        match *p {
            '0'..='9' => {
                let ch = iter.next().unwrap();
                s.push(ch);
            }
            '.' => {
                if prd {
                    return Err("invalid number value".to_string());
                }
                let ch = iter.next().unwrap();
                s.push(ch);
                prd = true;
            }
            _ => break,
        }
    }
    if s.chars().collect::<Vec<char>>()[s.len() - 1] == '.' {
        return Err(String::from("invalid number value"));
    }

    Ok(s.parse().unwrap())
}

fn tokenize_str(iter: &mut Peekable<Chars<'_>>) -> Result<String, String> {
    let mut s = String::from("");
    while let Some(p) = iter.peek() {
        match p {
            '"' => {
                iter.next().unwrap();
                break;
            }
            _ => {
                let ch = iter.next().unwrap();
                s.push(ch);
            }
        }
    }
    Ok(s)
}

fn tokenize_true(iter: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    if let Some(p) = iter.peek()
        && *p == 'r'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 'u'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 'e'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    Ok(Token::True)
}

fn tokenize_false(iter: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    if let Some(p) = iter.peek()
        && *p == 'a'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 'l'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 's'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 'e'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    Ok(Token::False)
}

fn tokenize_null(iter: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    if let Some(p) = iter.peek()
        && *p == 'u'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 'l'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    if let Some(p) = iter.peek()
        && *p == 'l'
    {
        iter.next().unwrap();
    } else {
        return Err(String::from("invalid bool token"));
    }

    Ok(Token::Null)
}

fn tokenize(json: String) -> Result<Vec<Token>, String> {
    let mut tokens = vec![];

    let mut iter = json.chars().into_iter().peekable();
    while let Some(c) = iter.next() {
        if c == ' ' {
            continue;
        }
        let token = match c {
            '{' => Token::BraceOpen,
            '}' => Token::BraceClose,
            '[' => Token::BracketOpen,
            ']' => Token::BracketClose,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '0'..='9' => Token::Number(tokenize_number(c, &mut iter)?),
            '"' => Token::String(tokenize_str(&mut iter)?),
            't' => tokenize_true(&mut iter)?,
            'f' => tokenize_false(&mut iter)?,
            'n' => tokenize_null(&mut iter)?,
            _ => return Err(String::from("invalid character")),
        };
        tokens.push(token);
    }
    Ok(tokens)
}

#[derive(Debug, Clone)]
enum JSON {
    Object(HashMap<String, JSON>),
    String(String),
    Array(Vec<JSON>),
    Number(f64),
    Bool(bool),
    Null,
}

fn parse_object<'a, I>(iter: &mut Peekable<I>) -> JSON
where
    I: Iterator<Item = &'a Token>,
{
    let mut map: HashMap<String, JSON> = HashMap::new();

    while let Some(p) = iter.peek() {
        if let Token::String(key) = *p {
            iter.next().unwrap();
            let colon = iter.next().unwrap();
            if *colon != Token::Colon {
                panic!("invalid json");
            }
            let value = parse_value(iter).unwrap();
            map.insert(key.clone(), value);
        } else if let Token::Comma = *p {
            iter.next().unwrap();
            continue;
        } else if let Token::BraceClose = *p {
            iter.next().unwrap();
            break;
        }
    }
    JSON::Object(map)
}

fn parse_arr<'a, I>(iter: &mut Peekable<I>) -> JSON
where
    I: Iterator<Item = &'a Token>,
{
    let mut arr: Vec<JSON> = vec![];

    while let Some(v) = parse_value(iter) {
        arr.push(v);
        let next = iter.peek().unwrap();
        if *(*next) == Token::Comma {
            iter.next().unwrap();
            continue;
        } else if *(*next) == Token::BracketClose {
            iter.next().unwrap();
            break;
        } else {
            break;
        }
    }

    JSON::Array(arr)
}

fn parse_value<'a, I>(iter: &mut Peekable<I>) -> Option<JSON>
where
    I: Iterator<Item = &'a Token>,
{
    match iter.next().unwrap() {
        Token::BracketOpen => Some(parse_arr(iter)),
        Token::BraceOpen => Some(parse_object(iter)),
        Token::String(s) => Some(JSON::String(s.clone())),
        Token::Number(n) => Some(JSON::Number(*n)),
        Token::True => Some(JSON::Bool(true)),
        Token::False => Some(JSON::Bool(false)),
        Token::Null => Some(JSON::Null),
        _ => todo!(),
    }
}

fn parse_json(s: String) -> JSON {
    let tokens = tokenize(s).unwrap();
    let mut iter = tokens.iter().peekable();

    match *iter.next().unwrap() {
        Token::BraceOpen => parse_object(&mut iter),
        Token::BracketOpen => parse_arr(&mut iter),
        _ => todo!(),
    }
}

fn main() {
    //let hp_str = fs::read_to_string("./haversine_pairs.json").unwrap();
    
    let js = String::from("[\"yeah\", true, false, null]");
    let json = parse_json(js.clone());
    println!("{:?}", json);
}
