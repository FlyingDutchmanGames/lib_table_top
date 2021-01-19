use lib_table_top::common::deck::STANDARD_DECK;
use lib_table_top::games::solitaire;

#[test]
fn test_foo() {
    let _game = solitaire::traditional::new(STANDARD_DECK);
    assert_eq!(1, 1)
}
