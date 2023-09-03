use std::fmt;

pub struct DisplayOption<T>(pub Option<T>);

impl<T> fmt::Display for DisplayOption<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(v) => fmt::Display::fmt(&v, f),
            None => f.write_str("none"),
        }
    }
}
