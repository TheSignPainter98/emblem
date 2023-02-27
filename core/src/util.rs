pub fn plural<T>(n: usize, singular: T, plural: T) -> T {
    match n {
        1 => singular,
        _ => plural,
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn plural() {
        assert_eq!("a", super::plural(1, "a", "b"));
        assert_eq!("b", super::plural(2, "a", "b"));
        assert_eq!("b", super::plural(0, "a", "b"));
    }
}
