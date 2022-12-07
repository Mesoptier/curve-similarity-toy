use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

#[wasm_bindgen]
pub fn sum(items: &[i32]) -> i32 {
    items.iter().sum()
}

#[wasm_bindgen]
pub fn max(items: &[i32]) -> Option<i32> {
    items.iter().max().copied()
}

#[wasm_bindgen]
pub enum NumberEnum {
    Foo = 0,
    Bar = 1,
    Qux = 2,
}

#[wasm_bindgen]
pub fn get_enum(name: &str) -> Option<NumberEnum> {
    match name {
        "foo" => Some(NumberEnum::Foo),
        "bar" => Some(NumberEnum::Bar),
        "qux" => Some(NumberEnum::Qux),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(1, 2);
        assert_eq!(result, 3);
    }
}
