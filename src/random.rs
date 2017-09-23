#[cfg(not(test))]
pub use self::prod::random;

#[cfg(not(test))]
mod prod {
    use rand;
    pub fn random() -> u32 { rand::random() }
}

#[cfg(test)]
pub use self::test::random;

#[cfg(test)]
mod test {
    use test_util;
    pub fn random() -> u32 { test_util::random() }
}
