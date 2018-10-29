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
pub const RAND_TEST_CONST: u32 = 100;

#[cfg(test)]
mod test {
    use super::RAND_TEST_CONST;
    pub fn random() -> u32 { RAND_TEST_CONST }
}
