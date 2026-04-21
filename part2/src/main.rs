use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt::Display,
    fs,
    iter::Peekable,
    str::Chars,
    time::{Duration, Instant},
};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

    fn from_json(j: &JSON) -> Result<CoordPair, String> {
        if let JSON::Object(map) = j {
            let x0 = match map.get("x0").ok_or(String::from("expected x0"))? {
                JSON::Number(n) => *n,
                other => return Err(String::from(format!("unexpected JSON value: {:?}", other))),
            };
            let y0 = match map.get("y0").ok_or(String::from("expected x0"))? {
                JSON::Number(n) => *n,
                other => return Err(String::from(format!("unexpected JSON value: {:?}", other))),
            };
            let x1 = match map.get("x1").ok_or(String::from("expected x0"))? {
                JSON::Number(n) => *n,
                other => return Err(String::from(format!("unexpected JSON value: {:?}", other))),
            };
            let y1 = match map.get("y1").ok_or(String::from("expected x0"))? {
                JSON::Number(n) => *n,
                other => return Err(String::from(format!("unexpected JSON value: {:?}", other))),
            };
            Ok(CoordPair { x0, y0, x1, y1 })
        } else {
            Err(String::from(format!("unexpected JSON value: {:?}", j)))
        }
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

fn generate_haversine_io(set_len: usize, output: &str) {
    let pairs = CoordPair::rand_set_clustered(set_len);

    let mut haversines = vec![];
    for i in 0..pairs.len() {
        let p = &pairs[i];
        let haver = ref_haversine(p.x0, p.y0, p.x1, p.y1, EARTH_RADIUS);
        haversines.push(haver);
    }
    let avg_sum = haversines.iter().sum::<f64>() / set_len as f64;
    println!("Pair Count: {set_len}");
    println!("Average Sum: {avg_sum}");

    let pair_json = serde_json::json!({
        "pairs": serde_json::to_value(pairs).unwrap()
    });
    let pair_json = pair_json.to_string();
    let mut calcs = vec![];
    for i in 0..haversines.len() {
        let h = haversines[i];
        calcs.push(h.to_string());
    }
    let calc_str = calcs.join("\n");

    fs::write(format!("{output}.json"), pair_json).unwrap();
    fs::write(format!("{output}.f64"), calc_str).unwrap();
}

fn get_haversine_json(input: &str) -> String {
    fs::read_to_string(format!("{input}.json")).unwrap()
}

fn get_haversine_calcs(input: &str) -> Vec<f64> {
    let s = fs::read_to_string(format!("{input}.f64")).unwrap();
    s.split("\n")
        .map(|n| {
            let f: f64 = n.parse().unwrap();
            f
        })
        .collect::<Vec<f64>>()
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
            'e' => {
                iter.next().unwrap();
                let mut scin = String::from("e");
                let ch = iter.next().unwrap();
                if ch != '-' {
                    return Err(format!("expected '-'; got '{ch}'"));
                }
                scin.push(ch);
                s.push_str(scin.as_str());
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

fn get_token(iter: &mut Peekable<Chars<'_>>) -> Result<Option<Token>, String> {
    while let Some(c) = iter.next() {
        if c == ' ' || c == '\n' {
            continue;
        }
        let token = match c {
            '{' => Token::BraceOpen,
            '}' => Token::BraceClose,
            '[' => Token::BracketOpen,
            ']' => Token::BracketClose,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '0'..='9' | '-' => Token::Number(tokenize_number(c, iter)?),
            '"' => Token::String(tokenize_str(iter)?),
            't' => tokenize_true(iter)?,
            'f' => tokenize_false(iter)?,
            'n' => tokenize_null(iter)?,
            _ => {
                return Err(format!("invalid char {}", c.clone()));
            }
        };
        return Ok(Some(token));
    }
    Ok(None)
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

fn parse_object(iter: &mut Peekable<Chars<'_>>) -> Result<JSON, String> {
    let mut map: HashMap<String, JSON> = HashMap::new();

    while let Some(t) = get_token(iter)? {
        if let Token::String(key) = t {
            let colon = get_token(iter)?.ok_or(String::from("expected token"))?;
            if colon != Token::Colon {
                return Err(String::from(format!("expected ':', got {colon:?}")));
            }
            let value = parse_value(iter)?.ok_or(String::from("expected json value"))?;
            map.insert(key.clone(), value);
        } else if t == Token::Comma {
            continue;
        } else if t == Token::BraceClose {
            break;
        }
    }
    Ok(JSON::Object(map))
}

fn parse_arr(iter: &mut Peekable<Chars<'_>>) -> Result<JSON, String> {
    let mut arr: Vec<JSON> = vec![];
    while let Some(v) = parse_value(iter)? {
        arr.push(v);
        let next = get_token(iter)?.ok_or(String::from("expected token"))?;
        match next {
            Token::Comma => continue,
            Token::BracketClose => break,
            _ => return Err(format!("invalid token: {:?}", next)),
        }
    }

    Ok(JSON::Array(arr))
}

fn parse_value(iter: &mut Peekable<Chars<'_>>) -> Result<Option<JSON>, String> {
    let token = get_token(iter)?;
    if token == None {
        return Ok(None);
    }
    let j = match token {
        None => None,
        Some(t) => match t {
            Token::BracketOpen => Some(parse_arr(iter)?),
            Token::BraceOpen => Some(parse_object(iter)?),
            Token::String(s) => Some(JSON::String(s.clone())),
            Token::Number(n) => Some(JSON::Number(n)),
            Token::True => Some(JSON::Bool(true)),
            Token::False => Some(JSON::Bool(false)),
            Token::Null => Some(JSON::Null),
            _ => None,
        },
    };
    Ok(j)
}

fn parse_json(s: &str) -> Result<Option<JSON>, String> {
    let mut iter = s.chars().into_iter().peekable();
    parse_value(&mut iter)
}

fn parse_pairs(s: &str) -> Result<Vec<CoordPair>, String> {
    let j = parse_json(s)?.ok_or(String::from("expected json: got None"))?;

    let mut pairs = vec![];

    let map = match j {
        JSON::Object(o) => o,
        other => return Err(String::from(format!("invalid json: {other:?}"))),
    };

    let arr = match map.get("pairs").ok_or(String::from("expected \"arr\""))? {
        JSON::Array(a) => a,
        other => return Err(String::from(format!("unexpected JSON value: {:?}", other))),
    };

    for val in arr {
        pairs.push(CoordPair::from_json(val)?)
    }

    Ok(pairs)
}

fn float_eq(a: f64, b: f64) -> bool {
    a - b < f64::EPSILON
}

fn compare_results(a: &[f64], b: &[f64]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        let r = a[i];
        let l = b[i];
        if !float_eq(r, l) {
            println!("mismatch at index: {}; {} != {}", i, r, l);
            return false;
        }
    }
    true
}

struct Results {
    input_size: usize,
    pair_count: usize,
    results_match: bool,
    results_avg: f64,
    ref_avg: f64,
    difference: f64,
    json_parse_time: Duration,
    haversine_calc_time: Duration,
}

impl Results {
    fn json_parse_time(&self) -> String {
        let dur = self.json_parse_time;

        let ns = dur.as_nanos();
        if ns < 1000 {
            return format!("{ns}ns");
        }

        let us = dur.as_micros();
        if us < 1000 {
            return format!("{us}us");
        }

        let ms = dur.as_millis();
        if ms < 1000 {
            return format!("{ms}ms");
        }

        let s = dur.as_secs_f64();
        return format!("{s}s");
    }

    fn haversine_calc_time(&self) -> String {
        let dur = self.haversine_calc_time;

        let ns = dur.as_nanos();
        if ns < 1000 {
            return format!("{ns}ns");
        }

        let us = dur.as_micros();
        if us < 1000 {
            return format!("{us}us");
        }

        let ms = dur.as_millis();
        if ms < 1000 {
            return format!("{ms}ms");
        }

        let s = dur.as_secs_f64();
        return format!("{s}s");
    }
}

impl Display for Results {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = vec![
            format!("Input Size: {}", self.input_size),
            format!("Pair Count: {}", self.pair_count),
            format!("Results Match: {}", self.results_match),
            format!("Haversine Sum: {}", self.results_avg),
            format!("JSON Parse Time: {}", self.json_parse_time()),
            format!("Haversine Calc Time: {}", self.haversine_calc_time()),
            String::from(""),
            String::from("Validation:"),
            format!("Reference Sum: {}", self.ref_avg),
            format!("Difference: {}", self.difference),
        ]
        .join("\n");
        write!(f, "{s}")
    }
}

// bin gen <num> <output>
fn generate(args: &[String]) -> Result<(), Box<dyn Error>> {
    assert_eq!(args.len(), 4);
    let num: usize = args[2].parse().unwrap();
    let output = args[3].clone();

    generate_haversine_io(num, &output);

    Ok(())
}

// bin ref <input>
// looks for <input>.json and <input>.f64
fn ref_alg(args: &[String]) -> Result<(), Box<dyn Error>> {
    assert_eq!(args.len(), 3);
    let input = args[2].clone();

    let hj = get_haversine_json(&input);
    let ref_results = get_haversine_calcs(&input);
    let mut results = vec![];

    let json_start = Instant::now();
    let pairs = parse_pairs(&hj)?;
    let json_parse_time = json_start.elapsed();

    let calc_start = Instant::now();
    for p in &pairs {
        results.push(ref_haversine(p.x0, p.y0, p.x1, p.y1, EARTH_RADIUS));
    }
    let haversine_calc_time = calc_start.elapsed();
    let results_match = compare_results(&ref_results, &results);

    let ref_avg = ref_results.iter().sum::<f64>() / ref_results.len() as f64;
    let results_avg = results.iter().sum::<f64>() / results.len() as f64;

    let r = Results {
        input_size: hj.as_bytes().len(),
        pair_count: pairs.len(),
        results_match,
        results_avg,
        ref_avg,
        difference: (ref_avg - results_avg).abs(),
        json_parse_time,
        haversine_calc_time,
    };
    println!("{r}");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() == 1 {
        println!("missing args");
        return Ok(());
    }
    match args[1].as_str() {
        "gen" => generate(&args)?,
        "ref" => ref_alg(&args)?,
        _ => return Err(format!("invalid argument {}", args[1]).into()),
    }
    Ok(())
}
