pub trait Key: Sized {
    fn build(&self) -> String;

    fn then<T: Key>(self, other: T) -> impl Key;
}

impl Key for String {
    fn build(&self) -> String {
        self.clone()
    }

    fn then<T: Key>(self, other: T) -> impl Key {
        (self, other)
    }
}

impl Key for &str {
    fn build(&self) -> String {
        self.to_string()
    }

    fn then<T: Key>(self, other: T) -> impl Key {
        (self, other)
    }
}

impl<A: Key, B: Key> Key for (A, B) {
    fn build(&self) -> String {
        format!("{}.{}", self.0.build(), self.1.build())
    }

    fn then<T: Key>(self, other: T) -> impl Key {
        (self, other)
    }
}

impl<A: Key> Key for Vec<A> {
    fn build(&self) -> String {
        self.iter()
            .map(|i| i.build())
            .fold("".to_owned(), |acc, i| format!("{acc}.{i}"))
    }

    fn then<T: Key>(self, other: T) -> impl Key {
        (self, other)
    }
}

impl<A: Key> Key for &[A] {
    fn build(&self) -> String {
        self.iter()
            .map(|i| i.build())
            .fold(String::new(), |acc, s| {
                if acc.is_empty() {
                    s
                } else {
                    format!("{acc}.{s}")
                }
            })
    }

    fn then<T: Key>(self, other: T) -> impl Key {
        (self, other)
    }
}
