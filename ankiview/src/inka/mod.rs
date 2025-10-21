pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod cli;

#[cfg(test)]
mod tests {
    #[test]
    fn given_empty_project_when_building_then_compiles() {
        assert!(true);
    }
}
