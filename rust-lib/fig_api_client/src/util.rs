use serde::de::Unexpected;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
};

/// Represents a json object with the type signature `T | [T]`;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ElementOrList<T> {
    Element(T),
    List(Vec<T>),
}

impl<T> ElementOrList<T> {
    pub fn iter(&self) -> ElementOrListIterator<T> {
        match self {
            ElementOrList::Element(e) => ElementOrListIterator::Element(Some(e)),
            ElementOrList::List(l) => ElementOrListIterator::List(l.iter()),
        }
    }
}

impl<T> IntoIterator for ElementOrList<T> {
    type IntoIter = ElementOrListIntoIter<Self::Item>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ElementOrList::Element(e) => ElementOrListIntoIter::Element(Some(e)),
            ElementOrList::List(l) => ElementOrListIntoIter::List(l.into_iter()),
        }
    }
}

pub enum ElementOrListIterator<'a, T> {
    Element(Option<&'a T>),
    List(std::slice::Iter<'a, T>),
}

impl<'a, T> Iterator for ElementOrListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ElementOrListIterator::Element(e) => e.take(),
            ElementOrListIterator::List(l) => l.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ElementOrListIterator::Element(Some(_)) => (1, Some(1)),
            ElementOrListIterator::Element(None) => (0, Some(0)),
            ElementOrListIterator::List(l) => l.size_hint(),
        }
    }
}

impl<T> ExactSizeIterator for ElementOrListIterator<'_, T> {}

#[derive(Debug, Clone)]
pub enum ElementOrListIntoIter<T> {
    Element(Option<T>),
    List(std::vec::IntoIter<T>),
}

impl<T> Iterator for ElementOrListIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ElementOrListIntoIter::Element(e) => e.take(),
            ElementOrListIntoIter::List(l) => l.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ElementOrListIntoIter::Element(Some(_)) => (1, Some(1)),
            ElementOrListIntoIter::Element(None) => (0, Some(0)),
            ElementOrListIntoIter::List(l) => l.size_hint(),
        }
    }
}

impl<T> ExactSizeIterator for ElementOrListIntoIter<T> {}

pub fn string_as_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).and_then(|s| {
        s.parse()
            .map_err(|_| serde::de::Error::invalid_value(Unexpected::Other("invalid u64"), &"valid u64"))
    })
}

pub fn string_as_option_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer).and_then(|s: Option<String>| {
        if let Some(s) = s {
            match s.parse() {
                Ok(s) => Ok(Some(s)),
                Err(_) => Err(serde::de::Error::invalid_value(
                    Unexpected::Other("invalid u64"),
                    &"valid u64",
                )),
            }
        } else {
            Ok(None)
        }
    })
}

pub fn string_as_vec_u64<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::deserialize(deserializer).and_then(|v: Vec<String>| {
        v.into_iter()
            .map(|s| {
                s.parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Other("invalid u64"), &"valid u64"))
            })
            .collect()
    })
}
