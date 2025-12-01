#[cfg(test)]
mod tests {
    #[test]
    fn slice_start() {
        let string = String::from("This is a string");
        let slice = &string[3..5];

        let slice_index;

        unsafe {
            slice_index = slice.as_ptr().offset_from(string.as_ptr());
        }

        assert_eq!(slice_index, 3);
    }

    #[test]
    fn float_to_str() {
      let n = 12.23;

      assert_eq!(&format!("{n}"), "12.23")
    }
}
