#[macro_export]
macro_rules! map_try {
  [ $expr:expr, $pat:pat => $mapper:expr ] => ({
    match $expr {
        Ok(val) => val,
        $pat => {
            let __e = $mapper;

            return Err(Into::into(__e));
        }
    }
  });
}
