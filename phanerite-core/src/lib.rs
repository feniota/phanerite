pub mod auth;
pub mod download;
pub mod error;
pub mod io;
pub mod java;
pub mod storage;
pub mod utils;
pub mod version;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
