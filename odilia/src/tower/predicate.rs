pub trait Predicate<T> {
	fn test(x: &T) -> bool;
}
