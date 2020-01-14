/// Parse and modify /etc/nix.conf in a comment/structure preserving fashion.
/// See test_nix_conf for usage.
use blake2::{Blake2b, Digest};
use generic_array::GenericArray;
#[cfg(test)]
use proptest::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::str::Lines;
use typenum::U64;

type Digested = GenericArray<u8, U64>;
fn is_comment(line: &str) -> bool {
    line.starts_with('#')
}
fn setting(s: &str) -> Option<(&str, &str)> {
    if is_comment(s) {
        None
    } else {
        match s.find('=') {
            None => None,
            Some(i) => {
                let (k, v) = s.split_at(i);
                Some((k.trim_end(), v.get(1..).unwrap().trim()))
            }
        }
    }
}
fn is_blank(line: &str) -> bool {
    line.find(|c: char| !c.is_ascii_whitespace()) == None
}

fn hashit(s: &str) -> Digested {
    let mut hasher = Blake2b::new();
    hasher.input(s.as_bytes());
    hasher.result()
}
fn format_pair(key: &str, value: &str) -> String {
    if value == "" {
        format!("{} =", key)
    } else {
        format!("{} = {}", key, value)
    }
}
#[cfg(test)]
#[test]
fn test_helpers() {
    assert!(is_comment("#comment"));
    assert!(!is_comment("non-comment"));
    assert!(!is_comment("   #non-comment"));
    assert!(!is_comment("a=b   #non-comment"));
    assert!(!is_blank("#a b c #$@#$=b   #non-comment"));
    assert!(is_blank(""));
    assert!(is_blank(" "));
    assert!(is_blank(" \t"));
    assert!(!is_blank(" \t,"));
    assert_eq!(
        setting("a=b   #non-comment"),
        Some(("a", "b   #non-comment"))
    );
    assert_eq!(
        setting("a= b   #non-comment"),
        Some(("a", "b   #non-comment"))
    );
    assert_eq!(
        setting("a  = b   #non-comment "),
        Some(("a", "b   #non-comment"))
    );
    assert_eq!(
        setting("a b c #$@#$=b   #non-comment"),
        Some(("a b c #$@#$", "b   #non-comment"))
    );
    assert_eq!(setting("#a b c #$@#$=b   #non-comment"), None);
    assert!(hashit("foo") == hashit("foo"));
    assert!(hashit("foo") != hashit("bar"));
}
type StartEnd = (usize, usize);

/// Make Peekable<Lines> less annoying.
struct ConfLines<'a> {
    lines: std::iter::Peekable<Lines<'a>>,
    pos: usize,
}
impl<'a> ConfLines<'a> {
    fn new(conf: &'a str) -> Self {
        let lines = conf.lines().peekable();
        Self { lines, pos: 0 }
    }
    fn peek(&mut self) -> Option<&str> {
        match self.lines.peek() {
            None => None,
            Some(&s) => Some(s),
        }
    }
    fn next(&mut self) -> Option<&str> {
        match self.lines.next() {
            None => None,
            x => {
                self.pos += 1;
                x
            }
        }
    }
}

pub struct CommentedSetting {
    pub value: String,
    pub comment: String,
}

impl From<&str> for CommentedSetting {
    fn from(value: &str) -> Self {
        CommentedSetting {
            value: value.to_string(),
            comment: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct NixConf {
    orig_hash: Digested,
    lines: Vec<String>,
    offsets: HashMap<String, StartEnd>,
    d: HashMap<String, String>,
}
impl fmt::Display for NixConf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_impl())
    }
}
/// Comments/structure preserving nix.conf reader/writer with
/// a HashMap like interface.
///
/// Setting or deleting a value also clears the comments associated with it.
/// Trailing whitespace and space around = are normalized, other than that files
/// should roundtrip if no modifications.
///
impl NixConf {
    pub fn read_from_str(conf: &str) -> Result<NixConf, String> {
        let mut lines: Vec<String> = Vec::new();
        let mut raw_lines = ConfLines::new(conf);
        let mut offsets = HashMap::new();
        let mut d = HashMap::new();
        while let Some(_line) = raw_lines.peek() {
            if let Err(x) = Self::parse1(&mut raw_lines, &mut lines, &mut offsets, &mut d) {
                return Err(x);
            }
        }
        Ok(Self {
            orig_hash: hashit(conf),
            lines,
            offsets,
            d,
        })
    }
    pub fn has_changed(&self) -> bool {
        self.orig_hash != hashit(&self.to_string())
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.d.get(key)
    }
    pub fn insert(&mut self, key: String, commented_value: CommentedSetting) -> Option<String> {
        let len = self.lines.len();
        let sv = Some(commented_value.value.clone());
        let to_insert = commented_value
            .comment
            .lines()
            .map(|l| format!("# {}", l))
            .chain(sv.iter().map(|v| format_pair(&key, &v)));
        let l3 = (len, len);
        let (start, end) = self.offsets.get(&key).unwrap_or(&l3);
        let (start, end) = (*start, *end);
        self.lines.splice(start..end, to_insert);
        let delta = self.lines.len() as isize - len as isize;
        let new_end = (end as isize + delta) as usize;
        let ans = self.d.insert(key.clone(), commented_value.value);
        self.offsets.insert(key, (start, new_end));
        self.adjust_offsets(start, delta);
        ans
    }
    pub fn remove(&mut self, key: String) -> Option<String> {
        let (start, end) = self.offsets.remove(&key)?;
        let delta = end as isize - start as isize;
        let ans = self.d.remove(&key);
        self.lines.splice(start..end, vec![]);
        self.adjust_offsets(start, delta);
        ans
    }
    fn adjust_offsets(&mut self, start: usize, delta: isize) {
        let add = |u, i| (u as isize + i) as usize;
        for (_, v) in self.offsets.iter_mut() {
            let (s, e) = *v;
            if s > start {
                *v = (add(s, delta), add(e, delta));
            }
        }
    }
    fn to_string_impl(&self) -> String {
        self.lines.join("\n").to_string() + "\n"
    }

    fn parse1(
        iter: &mut ConfLines,
        lines: &mut Vec<String>,
        offsets: &mut HashMap<String, StartEnd>,
        d: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let start_pos = iter.pos;
        while iter.peek().map_or(false, is_blank) {
            iter.next();
            lines.push("".to_string());
        }
        while iter.peek().map_or(false, is_comment) {
            lines.push(iter.next().unwrap().trim_end().to_string());
        }
        let setting_pos = iter.pos;
        match iter.peek() {
            // EOF
            None => Ok(()),
            // Blank section
            Some(blank) if is_blank(blank) => loop {
                iter.next();
                if iter.peek().map(is_blank) != Some(true) {
                    return Ok(());
                }
            },
            // Must be a setting
            Some(s) => {
                if let Some((k, v)) = setting(s) {
                    if d.contains_key(k) {
                        return Err(format!(
                            "{}: Duplicate entry for `{}` (first seen on {}): {}",
                            setting_pos + 1,
                            k,
                            offsets.get(k).unwrap().1,
                            s
                        ));
                    }
                    lines.push(format_pair(k, v));
                    d.insert(k.to_string(), v.to_string());
                    offsets.insert(k.to_string(), (start_pos, setting_pos + 1));
                    iter.next();
                    Ok(())
                } else {
                    Err(format!("{}: Malformed line: {}", setting_pos + 1, s))
                }
            }
        }
    }
}
#[cfg(test)]
#[test]
fn test_nix_conf_canonicalization() {
    let canonical = "max-jobs = auto\n";
    {
        let nc = NixConf::read_from_str(canonical).unwrap();
        assert!(!nc.has_changed());
    }
    {
        let nc = NixConf::read_from_str("max-jobs= auto").unwrap();
        assert!(nc.has_changed());
        assert_eq!(nc.to_string_impl(), canonical);
    }
    let canonical_unset = "something =\n";
    {
        let nc = NixConf::read_from_str(canonical_unset).unwrap();
        println!("<<{}>>", nc.to_string_impl());
        assert!(!nc.has_changed());
    }
    {
        let nc = NixConf::read_from_str("something=  \n").unwrap();
        assert!(nc.has_changed());
        assert_eq!(nc.to_string_impl(), canonical_unset);
    }
}
#[cfg(test)]
#[test]
fn test_nix_conf() {
    let conf = "\
max-jobs = auto

# Use all available CPU cores in the system
# Pass -jN to build jobs where N is the number of cores
cores = 0

bogus = value
# Since 2.2, Nix has sandboxing enabled by default
sandbox = true
# Make our impure builds work by letting them access the following
extra-sandbox-paths = /lib /lib64 /usr/bin

# Disable signatures as we trust the transport and storage
require-sigs = false
# First download from our own binary-cache, then upstream
substituters = s3://my-awesome-nix-cache https://cache.nixos.org
";
    let mut nc = NixConf::read_from_str(conf).unwrap();
    assert!(!nc.has_changed());
    // make an insert with the same value without comment (where there was none)
    assert_eq!(nc.get("max-jobs"), Some(&"auto".to_string()));
    nc.insert("max-jobs".to_string(), "auto".into());
    assert_eq!(nc.get("max-jobs"), Some(&"auto".to_string()));
    assert!(!nc.has_changed());
    // make an insert that changes neither comment nor value
    assert_eq!(nc.get("sandbox"), Some(&"true".to_string()));
    nc.insert(
        "sandbox".to_string(),
        CommentedSetting {
            value: "true".into(),
            comment: "Since 2.2, Nix has sandboxing enabled by default".into(),
        },
    );
    assert_eq!(nc.get("sandbox"), Some(&"true".to_string()));
    assert!(!nc.has_changed());
    // make an insert that changes the comment
    assert_eq!(nc.get("cores"), Some(&"0".to_string()));
    nc.insert(
        "cores".to_string(),
        CommentedSetting {
            value: "0".to_string(),
            comment: "Use all available CPU cores\non this system".to_string(),
        },
    );
    assert_eq!(nc.get("cores"), Some(&"0".to_string()));
    assert!(nc.has_changed());
    nc = NixConf::read_from_str(&nc.to_string_impl()).unwrap();
    assert_eq!(nc.get("bogus"), Some(&"value".to_string()));
    // remove a value
    nc.remove("bogus".to_string());
    assert_eq!(nc.get("bogus"), None);
    assert!(nc.has_changed());
    nc.insert("cores".to_string(), "8".into());
    assert_eq!(nc.get("cores"), Some(&"8".to_string()));
    // add a new value
    assert_eq!(nc.get("min-free"), None);
    nc.insert("min-free".to_string(), "0".into());
    assert_eq!(nc.get("min-free"), Some(&"0".to_string()));
    assert_eq!(
        nc.to_string(),
        "\
max-jobs = auto
cores = 8
# Since 2.2, Nix has sandboxing enabled by default
sandbox = true
# Make our impure builds work by letting them access the following
extra-sandbox-paths = /lib /lib64 /usr/bin

# Disable signatures as we trust the transport and storage
require-sigs = false
# First download from our own binary-cache, then upstream
substituters = s3://my-awesome-nix-cache https://cache.nixos.org
min-free = 0
"
    );
}
#[cfg(test)]
proptest! {
    #[test]
    fn single_clean_property_roundtrips(s in r#"(?:# .*\S\n)*\w+ = \S.*\S\n"#) {
        let nc = NixConf::read_from_str(&s).unwrap();
        assert_eq!(s, nc.to_string_impl());
    }
    #[test]
    fn single_unclean_property_does_not_roundtrips(s in r#"(?:#.*\S\n)*(?:\w+[^ \n]*=[^\n]*|\w.*=[^ \n].*)\S\n"#) {
        let nc = NixConf::read_from_str(&s).unwrap();
        assert!(s != nc.to_string_impl());
    }
}
