use termitype::generator::Generator;

#[test]
fn test_generator_instance() {
    let generator =
        Generator::new("assets/test_words.txt").expect("Failed to load test words file");
    assert!(!generator.words.is_empty())
}

#[test]
fn test_generate_text() {
    let generator =
        Generator::new("assets/test_words.txt").expect("Failed to load test words file");
    let text = generator.generate(10);
    let word_count = text.split_whitespace().count();
    assert_eq!(word_count, 10);
}
