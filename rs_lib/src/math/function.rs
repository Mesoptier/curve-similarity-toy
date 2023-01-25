pub trait Function<'f, T> {
    type Output;

    fn eval(&'f self, x: T) -> Self::Output;
}
