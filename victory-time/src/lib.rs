mod timecode;
mod timepoint;
mod timespan;
pub use timecode::*;
pub use timespan::*;
pub use timepoint::*;

#[cfg(test)]
mod tests_integration {
    use super::*;
    #[test]
    fn test_measure_time() {
        let start = Timepoint::now();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let end = Timepoint::now();
        let duration = end - start;
        println!("Duration: {:?}", duration.ms());
        assert!(duration.ms() >= 900.0);
    }
}
