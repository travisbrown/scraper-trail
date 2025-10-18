use std::{borrow::Cow, marker::PhantomData};

#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Empty values")]
    Empty,
}

/// A set of values for a response header.
///
/// Typically each header name will map to a single value, but the same name may appear more than
/// once, so we wish to handle that case, while still making it convenient to work with singleton
/// values.
#[derive(Clone, Debug, Eq, PartialEq, bounded_static_derive_more::ToStatic)]
pub struct MultiValue<'a> {
    pub first: Cow<'a, str>,
    rest: Option<Vec<Cow<'a, str>>>,
}

impl<'a> MultiValue<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(value: S) -> Self {
        Self {
            first: value.into(),
            rest: None,
        }
    }

    pub fn push<S: Into<Cow<'a, str>>>(&mut self, value: S) {
        match &mut self.rest {
            None => {
                self.rest = Some(vec![value.into()]);
            }
            Some(rest) => {
                rest.push(value.into());
            }
        }
    }

    #[must_use]
    pub fn iter(&'a self) -> Iter<'a> {
        Iter {
            first: Some(self.first.clone()),
            rest: self.rest.as_ref().map(|rest| rest.iter()),
        }
    }
}

impl<'a> AsRef<Cow<'a, str>> for MultiValue<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.first
    }
}

impl<'a, S: Into<Cow<'a, str>>> TryFrom<Vec<S>> for MultiValue<'a> {
    type Error = Error;

    fn try_from(mut value: Vec<S>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(Error::Empty)
        } else {
            let first = value.remove(0).into();
            let rest = if value.is_empty() {
                None
            } else {
                Some(value.into_iter().map(std::convert::Into::into).collect())
            };

            Ok(Self { first, rest })
        }
    }
}

impl<'a> IntoIterator for &'a MultiValue<'a> {
    type Item = std::borrow::Cow<'a, str>;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a> {
    first: Option<Cow<'a, str>>,
    rest: Option<std::slice::Iter<'a, Cow<'a, str>>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| {
            self.rest.take().and_then(|mut rest| {
                let next = rest.next().cloned();
                self.rest = Some(rest);
                next
            })
        })
    }
}

impl<'a, 'de: 'a> serde::de::Deserialize<'de> for MultiValue<'a> {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        const EXPECTED: &str = "one or more header values";

        struct MultiValueVisitor<'a> {
            _lifetime: PhantomData<&'a ()>,
        }

        impl<'a, 'de: 'a> serde::de::Visitor<'de> for MultiValueVisitor<'a> {
            type Value = MultiValue<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(EXPECTED)
            }

            fn visit_borrowed_str<E: serde::de::Error>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E> {
                Ok(MultiValue::new(v))
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut result: Option<MultiValue<'a>> = None;

                while let Some(value) = seq.next_element::<Cow<'a, str>>()? {
                    match result {
                        Some(ref mut multi_value) => {
                            multi_value.push(value);
                        }
                        None => {
                            result = Some(MultiValue::new(value));
                        }
                    }
                }

                result.ok_or_else(|| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Seq, &EXPECTED)
                })
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(MultiValue::new(v.to_string()))
            }
        }

        deserializer.deserialize_any(MultiValueVisitor {
            _lifetime: PhantomData,
        })
    }
}

impl serde::ser::Serialize for MultiValue<'_> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;

        match self.rest.as_ref() {
            Some(rest) => {
                let mut seq = serializer.serialize_seq(Some(rest.len() + 1))?;
                seq.serialize_element(&self.first)?;

                for element in rest {
                    seq.serialize_element(&element)?;
                }

                seq.end()
            }
            None => self.first.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::multi_value::MultiValue;

    #[derive(Debug, Eq, PartialEq, serde::Deserialize)]
    struct Test<'a> {
        #[serde(borrow)]
        header_values: super::MultiValue<'a>,
    }

    #[test]
    fn deserialize_multi_value() -> Result<(), Box<dyn std::error::Error>> {
        let singleton_example = r#"{ "header_values": "test" }"#;
        let multi_example = r#"{ "header_values": ["foo", "bar", "baz"] }"#;

        let singleton_example_parsed = serde_json::from_str::<Test<'_>>(singleton_example)?;
        let multi_example_parsed = serde_json::from_str::<Test<'_>>(multi_example)?;

        let singleton_example_expected = Test {
            header_values: MultiValue::new("test"),
        };

        let multi_example_expected = Test {
            header_values: vec!["foo", "bar", "baz"].try_into()?,
        };

        assert_eq!(singleton_example_parsed, singleton_example_expected);
        assert_eq!(multi_example_parsed, multi_example_expected);
        Ok(())
    }

    #[test]
    fn iter() -> Result<(), Box<dyn std::error::Error>> {
        let singleton_example = MultiValue::new("test");

        let multi_example: MultiValue<'_> = vec!["foo", "bar", "baz"].try_into()?;

        assert_eq!(singleton_example.iter().collect::<Vec<_>>(), vec!["test"]);
        assert_eq!(
            multi_example.iter().collect::<Vec<_>>(),
            vec!["foo", "bar", "baz"]
        );
        Ok(())
    }
}
