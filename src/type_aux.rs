

pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}


pub fn type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}
