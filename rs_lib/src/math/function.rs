pub trait Function<'f, T> {
    type Output;

    fn eval(&'f self, x: T) -> Self::Output;
}

impl<'f, T, O, F> Function<'f, T> for F
where
    F: Fn(T) -> O,
{
    type Output = O;

    fn eval(&'f self, x: T) -> Self::Output {
        self(x)
    }
}
