use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum PropValue {
    Integer(i64),
    Float(f64),
    Vector((f32, f32, f32)),
    String(String),
    List(Vec<PropValue>),
    Proplist(Proplist),
    Void,
}

impl std::fmt::Display for PropValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            PropValue::Integer(i) => i.to_string(),
            PropValue::Float(f) => f.to_string(),
            PropValue::Vector(v) => format!("vector({},{},{})", v.0, v.1, v.2),
            PropValue::String(s) => format!("\"{}\"", s),
            PropValue::List(l) => format!(
                "[{}]",
                l.iter()
                    .map(|pv| pv.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            PropValue::Proplist(p) => p.to_string(),
            PropValue::Void => "".to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub struct Proplist {
    pub elements: HashMap<String, PropValue>,
}

impl std::fmt::Display for Proplist {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let elements_str = self
            .elements
            .iter()
            .map(|(k, v)| format!("#{}:{}", k, v))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "[{}]", elements_str)
    }
}

impl Proplist {
    pub fn new() -> Self {
        Proplist {
            elements: HashMap::new(),
        }
    }

    pub fn add_element(&mut self, key: &str, value: PropValue) {
        self.elements.insert(key.to_string(), value);
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let trimmed = input.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err("Proplist must start with '[' and end with ']'".to_string());
        }

        let end = trimmed.len().saturating_sub(1);
        let content = trimmed
            .get(1..end)
            .ok_or("Proplist must start with '[' and end with ']'")?;
        let elements_vec = split_elements(content)?;

        let mut elements = HashMap::new();
        for item in elements_vec {
            let parts: Vec<&str> = item.splitn(2, ':').collect();
            let key = if !parts.is_empty() {
                parts[0].trim()
            } else {
                return Err(format!("Invalid element: {}", item));
            };

            if !key.starts_with('#') {
                return Err(format!("Key must start with '#': {}", key));
            }
            let key = key[1..].to_string();

            let value = if parts.len() == 2 {
                let value_str = parts[1].trim();
                if value_str.is_empty() {
                    PropValue::Void
                } else {
                    PropValue::from_str(value_str)?
                }
            } else {
                PropValue::Void
            };

            elements.insert(key, value);
        }

        Ok(Proplist { elements })
    }

    /// Retrieves an element by its key.
    pub fn get_element(&self, key: &str) -> Option<&PropValue> {
        self.elements.get(key)
    }
}

fn split_elements(s: &str) -> Result<Vec<String>, String> {
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut depth: u32 = 0;
    let chars = s.chars().peekable();

    for c in chars {
        match c {
            '[' | '(' => {
                depth = depth.checked_add(1).ok_or("Depth overflow")?;
                current.push(c);
            }
            ']' | ')' => {
                if depth == 0 {
                    return Err(format!("Unmatched closing '{}'", c));
                }
                depth = depth.checked_sub(1).ok_or("Depth underflow")?;
                current.push(c);
            }
            ',' if depth == 0 => {
                elements.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    if depth != 0 {
        return Err("Unmatched opening bracket or parenthesis".to_string());
    }

    if !current.trim().is_empty() {
        elements.push(current.trim().to_string());
    }

    Ok(elements)
}

impl FromStr for PropValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_end = s.len().saturating_sub(1);
        if s.starts_with("vector(") && s.ends_with(')') {
            let inner = s.get(7..s_end).ok_or("Invalid vector format")?;
            let nums = inner
                .split(',')
                .map(|num| num.trim().parse::<f64>().map_err(|e| e.to_string()))
                .collect::<Result<Vec<f64>, String>>()?;
            if nums.len() != 3 {
                return Err("Vector must have exactly three components".to_string());
            }
            Ok(PropValue::Vector((
                nums[0] as f32,
                nums[1] as f32,
                nums[2] as f32,
            )))
        } else if s.starts_with('"') && s.ends_with('"') {
            let inner = s.get(1..s_end).ok_or("Invalid string format")?;
            Ok(PropValue::String(inner.to_string()))
        } else if let Ok(i) = s.parse::<i64>() {
            Ok(PropValue::Integer(i))
        } else if let Ok(f) = s.parse::<f64>() {
            Ok(PropValue::Float(f))
        } else if s.starts_with('[') && s.ends_with(']') {
            let inner = s.get(1..s_end).ok_or("Invalid list format")?;
            // Check if inner starts with '#', treat as Proplist
            if inner.trim_start().starts_with('#') {
                let proplist = Proplist::parse(s)?;
                Ok(PropValue::Proplist(proplist))
            } else {
                let list = split_elements(inner)?
                    .into_iter()
                    .map(|item| PropValue::from_str(&item))
                    .collect::<Result<Vec<PropValue>, String>>()?;
                Ok(PropValue::List(list))
            }
        } else {
            Err(format!("Unknown value type: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proplist() {
        let input = r#"[#l:vector(382.76, -95.33, 183.54),#r:vector(0.00, -44.36, 0.00),#ic:0,#gs:0,#as:"",#ar:0.15]"#;
        let proplist = Proplist::parse(input).expect("Failed to parse proplist");
        assert_eq!(proplist.elements.len(), 6);

        assert_eq!(
            proplist.elements.get("l"),
            Some(&PropValue::Vector((382.76, -95.33, 183.54)))
        );
        assert_eq!(
            proplist.elements.get("r"),
            Some(&PropValue::Vector((0.00, -44.36, 0.00)))
        );
        assert_eq!(proplist.elements.get("ic"), Some(&PropValue::Integer(0)));
        assert_eq!(proplist.elements.get("gs"), Some(&PropValue::Integer(0)));
        assert_eq!(
            proplist.elements.get("as"),
            Some(&PropValue::String("".to_string()))
        );
        assert_eq!(proplist.elements.get("ar"), Some(&PropValue::Float(0.15)));
    }

    #[test]
    fn test_parse_proplist_with_nested_list() {
        let input = r#"[#list:[1, 2.5, "three", vector(4.0, 5.0, 6.0)],#empty_list:[]]"#;
        let proplist = Proplist::parse(input).expect("Failed to parse proplist with nested list");
        assert_eq!(proplist.elements.len(), 2);

        assert_eq!(
            proplist.elements.get("list"),
            Some(&PropValue::List(vec![
                PropValue::Integer(1),
                PropValue::Float(2.5),
                PropValue::String("three".to_string()),
                PropValue::Vector((4.0, 5.0, 6.0)),
            ]))
        );

        assert_eq!(
            proplist.elements.get("empty_list"),
            Some(&PropValue::List(vec![]))
        );
    }

    #[test]
    fn test_parse_proplist_with_void() {
        let input = r#"[#enabled:,#disabled:0,#name:"Player"]"#;
        let proplist = Proplist::parse(input).expect("Failed to parse proplist with void");
        assert_eq!(proplist.elements.len(), 3);

        assert_eq!(proplist.elements.get("enabled"), Some(&PropValue::Void));
        assert_eq!(
            proplist.elements.get("disabled"),
            Some(&PropValue::Integer(0))
        );
        assert_eq!(
            proplist.elements.get("name"),
            Some(&PropValue::String("Player".to_string()))
        );
    }

    #[test]
    fn test_parse_proplist_with_nested_proplist_in_list() {
        let input = r#"[#main_list:[#nested1:123, #nested2:"value"], #number:456]"#;
        let proplist =
            Proplist::parse(input).expect("Failed to parse proplist with nested proplist in list");
        assert_eq!(proplist.elements.len(), 2);

        match proplist.elements.get("main_list") {
            Some(PropValue::Proplist(nested)) => {
                assert_eq!(
                    nested.elements.get("nested1"),
                    Some(&PropValue::Integer(123))
                );
                assert_eq!(
                    nested.elements.get("nested2"),
                    Some(&PropValue::String("value".to_string()))
                );
            }
            _ => panic!("Expected main_list to be a Proplist"),
        }

        assert_eq!(
            proplist.elements.get("number"),
            Some(&PropValue::Integer(456))
        );
    }

    #[test]
    fn test_parse_proplist_invalid() {
        let input = r#"[#invalid:unknown, #missing_quote:"value, #no_end_vector:vector(1.0, 2.0]"#;
        let result = Proplist::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_element() {
        let input = r#"[#key1:123,#key2:"value",#key3:vector(1.0,2.0,3.0)]"#;
        let proplist = Proplist::parse(input).expect("Failed to parse proplist");

        assert_eq!(proplist.get_element("key1"), Some(&PropValue::Integer(123)));
        assert_eq!(
            proplist.get_element("key2"),
            Some(&PropValue::String("value".to_string()))
        );
        assert_eq!(
            proplist.get_element("key3"),
            Some(&PropValue::Vector((1.0, 2.0, 3.0)))
        );
        assert_eq!(proplist.get_element("nonexistent"), None);
    }
}
