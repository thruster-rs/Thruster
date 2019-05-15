#[macro_export]
macro_rules! map_try {
  [ $expr:expr, $pat:pat => $mapper:block ] => ({
    match $expr {
        Ok(val) => val,
        $pat => {
            let __e = $mapper;

            return Err(__e);
        }
    }
  });
}
