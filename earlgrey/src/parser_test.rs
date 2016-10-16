use lexers::{Scanner, DelimTokenizer};
use types::{StateSet, GrammarBuilder, Grammar};
use parser::{EarleyParser, ParseError};
use trees::{one_tree, all_trees, Subtree};
use std::collections::HashSet;
use std::iter::FromIterator;


// Sum -> Sum + Mul | Mul
// Mul -> Mul * Pow | Pow
// Pow -> Num ^ Pow | Num
// Num -> Number | ( Sum )

fn grammar_math() -> Grammar {
    GrammarBuilder::new()
      // register some symbols
      .symbol("Sum")
      .symbol("Mul")
      .symbol("Pow")
      .symbol("Num")
      .symbol(("Number", |n: &str| {
          n.chars().all(|c| "1234567890".contains(c))
        }))
      .symbol(("[+-]", |n: &str| {
          n.len() == 1 && "+-".contains(n)
        }))
      .symbol(("[*/]", |n: &str| {
          n.len() == 1 && "*/".contains(n)
        }))
      .symbol(("[^]", |n: &str| { n == "^" }))
      .symbol(("(", |n: &str| { n == "(" }))
      .symbol((")", |n: &str| { n == ")" }))
      // add grammar rules
      .rule("Sum", &["Sum", "[+-]", "Mul"])
      .rule("Sum", &["Mul"])
      .rule("Mul", &["Mul", "[*/]", "Pow"])
      .rule("Mul", &["Pow"])
      .rule("Pow", &["Num", "[^]", "Pow"])
      .rule("Pow", &["Num"])
      .rule("Num", &["(", "Sum", ")"])
      .rule("Num", &["Number"])
      .into_grammar("Sum")
}

fn print_statesets(ss: &Vec<StateSet>) {
    for (idx, stateset) in ss.iter().enumerate() {
        println!("=== {} ===", idx);
        for item in stateset.iter() { println!("{:?}", item); }
    }
}

fn check_trees(trees: &Vec<Subtree>, expected: Vec<&str>) {
    assert_eq!(trees.len(), expected.len());
    let mut expect = HashSet::<&str>::from_iter(expected);
    for t in trees {
        let teststr = format!("{:?}", t);
        assert!(expect.remove(teststr.as_str()));
    }
    assert_eq!(0, expect.len());
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_badparse() {
    let mut input = DelimTokenizer::from_str("1+", "+*", false);
    let out = EarleyParser::new(grammar_math()).parse(&mut input);
    assert_eq!(out.unwrap_err(), ParseError::BadInput);
}

#[test]
fn test_partialparse() {
    let gb = GrammarBuilder::new()
      .symbol("Start")
      .symbol(("+", |n: &str| n == "+"))
      .rule("Start", &["+", "+"]);
    let mut input = DelimTokenizer::from_str("+++", "+", false);
    let out = EarleyParser::new(gb.into_grammar("Start")).parse(&mut input);
    assert_eq!(out.unwrap_err(), ParseError::BadInput);
}

#[test]
fn grammar_ambiguous() {
    // S -> SS | b
    let gb = GrammarBuilder::new()
      .symbol("S")
      .symbol(("b", |n: &str| n == "b"))
      .rule("S", &["S", "S"])
      .rule("S", &["b"]);
    // Earley's corner case that generates spurious trees for bbb
    let mut input = DelimTokenizer::from_str("b b b", " ", true);
    let p = EarleyParser::new(gb.into_grammar("S"));
    let ps = p.parse(&mut input).unwrap();
    // verify
    assert_eq!(ps.len(), 4);
    let trees = all_trees(p.g.start(), &ps);
    check_trees(&trees, vec![
        r#"SubT("S -> S S", [SubT("S -> S S", [SubT("S -> b", [Node("b", "b")]), SubT("S -> b", [Node("b", "b")])]), SubT("S -> b", [Node("b", "b")])])"#,
        r#"SubT("S -> S S", [SubT("S -> b", [Node("b", "b")]), SubT("S -> S S", [SubT("S -> b", [Node("b", "b")]), SubT("S -> b", [Node("b", "b")])])])"#,
    ]);
    println!("=== tree ===");
    for t in all_trees(p.g.start(), &ps) { println!("{:?}", t); }
}

#[test]
fn grammar_ambiguous_epsilon() {
    // S -> SSX | b
    // X -> <e>
    let gb = GrammarBuilder::new()
      .symbol("S")
      .symbol("X")
      .symbol(("b", |n: &str| n == "b"))
      .rule("S", &["S", "S", "X"])
      .rule("X", &[])
      .rule("S", &["b"]);
    // Earley's corner case that generates spurious trees for bbb
    let mut input = DelimTokenizer::from_str("b b b", " ", true);
    let p = EarleyParser::new(gb.into_grammar("S"));
    let ps = p.parse(&mut input).unwrap();
    assert_eq!(ps.len(), 4);
    let trees = all_trees(p.g.start(), &ps);
    check_trees(&trees, vec![
        r#"SubT("S -> S S X", [SubT("S -> S S X", [SubT("S -> b", [Node("b", "b")]), SubT("S -> b", [Node("b", "b")]), SubT("X -> ", [])]), SubT("S -> b", [Node("b", "b")]), SubT("X -> ", [])])"#,
        r#"SubT("S -> S S X", [SubT("S -> b", [Node("b", "b")]), SubT("S -> S S X", [SubT("S -> b", [Node("b", "b")]), SubT("S -> b", [Node("b", "b")]), SubT("X -> ", [])]), SubT("X -> ", [])])"#,
    ]);
}

#[test]
fn math_grammar_test() {
    let p = EarleyParser::new(grammar_math());
    let mut input = DelimTokenizer::from_str("1+(2*3-4)", "+*-/()", false);
    let ps = p.parse(&mut input).unwrap();
    assert_eq!(ps.len(), 10);
    let trees = all_trees(p.g.start(), &ps);
    check_trees(&trees, vec![
        r#"SubT("Sum -> Sum [+-] Mul", [SubT("Sum -> Mul", [SubT("Mul -> Pow", [SubT("Pow -> Num", [SubT("Num -> Number", [Node("Number", "1")])])])]), Node("[+-]", "+"), SubT("Mul -> Pow", [SubT("Pow -> Num", [SubT("Num -> ( Sum )", [Node("(", "("), SubT("Sum -> Sum [+-] Mul", [SubT("Sum -> Mul", [SubT("Mul -> Mul [*/] Pow", [SubT("Mul -> Pow", [SubT("Pow -> Num", [SubT("Num -> Number", [Node("Number", "2")])])]), Node("[*/]", "*"), SubT("Pow -> Num", [SubT("Num -> Number", [Node("Number", "3")])])])]), Node("[+-]", "-"), SubT("Mul -> Pow", [SubT("Pow -> Num", [SubT("Num -> Number", [Node("Number", "4")])])])]), Node(")", ")")])])])])"#,
    ]);
    assert_eq!(one_tree(p.g.start(), &ps), trees[0]);
}

#[test]
fn test_left_recurse() {
    // S -> S + N | N
    // N -> [0-9]
    let gb = GrammarBuilder::new()
      .symbol("S")
      .symbol("N")
      .symbol(("[+]", |n: &str| n == "+"))
      .symbol(("[0-9]", |n: &str| "1234567890".contains(n)))
      .rule("S", &["S", "[+]", "N"])
      .rule("S", &["N"])
      .rule("N", &["[0-9]"]);
    let p = EarleyParser::new(gb.into_grammar("S"));
    let mut input = DelimTokenizer::from_str("1+2", "+", false);
    let ps = p.parse(&mut input).unwrap();
    let tree = one_tree(p.g.start(), &ps);
    check_trees(&vec![tree], vec![
        r#"SubT("S -> S [+] N", [SubT("S -> N", [SubT("N -> [0-9]", [Node("[0-9]", "1")])]), Node("[+]", "+"), SubT("N -> [0-9]", [Node("[0-9]", "2")])])"#,
    ]);
}

#[test]
fn test_right_recurse() {
    // P -> N ^ P | N
    // N -> [0-9]
    let gb = GrammarBuilder::new()
      .symbol("P")
      .symbol("N")
      .symbol(("[^]", |n: &str| n == "^"))
      .symbol(("[0-9]", |n: &str| "1234567890".contains(n)))
      .rule("P", &["N", "[^]", "P"])
      .rule("P", &["N"])
      .rule("N", &["[0-9]"]);
    let p = EarleyParser::new(gb.into_grammar("P"));
    let mut input = DelimTokenizer::from_str("1^2", "^", false);
    let ps = p.parse(&mut input).unwrap();
    let tree = one_tree(p.g.start(), &ps);
    check_trees(&vec![tree], vec![
        r#"SubT("P -> N [^] P", [SubT("N -> [0-9]", [Node("[0-9]", "1")]), Node("[^]", "^"), SubT("P -> N", [SubT("N -> [0-9]", [Node("[0-9]", "2")])])])"#,
    ]);
}

#[test]
fn bogus_empty() {
    // A -> <empty> | B
    // B -> A
    let gb = GrammarBuilder::new()
      .symbol("A")
      .symbol("B")
      .rule("A", &vec![])
      .rule("A", &vec!["B"])
      .rule("B", &vec!["A"]);
    let g = gb.into_grammar("A");
    let p = EarleyParser::new(g);
    let mut input = DelimTokenizer::from_str("", "-", false);
    let ps = p.parse(&mut input).unwrap();
    // this generates an infinite number of parse trees, don't check/print them all
    check_trees(&vec![one_tree(p.g.start(), &ps)], vec![r#"SubT("A -> ", [])"#]);
}

#[test]
fn bogus_epsilon() {
    // Grammar for balanced parenthesis
    // P  -> '(' P ')' | P P | <epsilon>
    let gb = GrammarBuilder::new()
      .symbol("P")
      .symbol(("(", |l: &str| l == "("))
      .symbol((")", |l: &str| l == ")"))
      .rule("P", &["(", "P", ")"])
      .rule("P", &["P", "P"])
      .rule("P", &[]);
    let g = gb.into_grammar("P");
    let p = EarleyParser::new(g);
    let mut input = Scanner::from_buf("".split_whitespace()
                                      .map(|s| s.to_string()));
    let ps = p.parse(&mut input).unwrap();
    // this generates an infinite number of parse trees, don't check/print them all
    check_trees(&vec![one_tree(p.g.start(), &ps)], vec![r#"SubT("P -> ", [])"#]);
}

#[test]
fn grammar_example() {
    // Grammar for all words containing 'main'
    // Program   -> Letters 'm' 'a' 'i' 'n' Letters
    // Letters   -> oneletter Letters | <epsilon>
    // oneletter -> [a-zA-Z]
    let gb = GrammarBuilder::new()
      .symbol("Program")
      .symbol("Letters")
      .symbol(("oneletter", |l: &str| l.len() == 1 &&
               l.chars().next().unwrap().is_alphabetic()))
      .symbol(("m", |l: &str| l == "m"))
      .symbol(("a", |l: &str| l == "a"))
      .symbol(("i", |l: &str| l == "i"))
      .symbol(("n", |l: &str| l == "n"))
      .rule("Program", &["Letters", "m", "a", "i", "n", "Letters"])
      .rule("Letters", &["oneletter", "Letters"])
      .rule("Letters", &[]);
    let p = EarleyParser::new(gb.into_grammar("Program"));
    let mut input = Scanner::from_buf("containsmainword".chars().map(|c| c.to_string()));
    assert!(p.parse(&mut input).is_ok());
}

#[test]
fn math_ambiguous() {
    // E -> E + E | E * E | n
    let gb = GrammarBuilder::new()
      .symbol("E")
      .symbol(("+", |n: &str| n == "+"))
      .symbol(("*", |n: &str| n == "*"))
      .symbol(("n", |n: &str|
          n.chars().all(|c| "1234567890".contains(c))))
      .rule("E", &["E", "+", "E"])
      .rule("E", &["E", "*", "E"])
      .rule("E", &["n"]);
    // number of trees here should match Catalan numbers if same operator
    let p = EarleyParser::new(gb.into_grammar("E"));
    let mut input = DelimTokenizer::from_str("0*1*2*3*4*5", "*", false);
    let ps = p.parse(&mut input).unwrap();
    let trees = all_trees(p.g.start(), &ps);
    check_trees(&trees, vec![
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> n", [Node("n", "3")])]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "2")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "1")]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> n", [Node("n", "4")])]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])"#,
        r#"SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> E * E", [SubT("E -> n", [Node("n", "0")]), Node("*", "*"), SubT("E -> n", [Node("n", "1")])]), Node("*", "*"), SubT("E -> n", [Node("n", "2")])]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "3")]), Node("*", "*"), SubT("E -> E * E", [SubT("E -> n", [Node("n", "4")]), Node("*", "*"), SubT("E -> n", [Node("n", "5")])])])])"#,
    ]);
}

#[test]
fn math_various() {
    let p = EarleyParser::new(grammar_math());
    let inputs = vec![
        "1+2^3^4*5/6+7*8^9",
        "(1+2^3)^4*5/6+7*8^9",
        "1+2^3^4*5",
        "(1+2)*3",
    ];
    for input in inputs.iter() {
        println!("============ input: {}", input);
        let mut input = DelimTokenizer::from_str(input, "+*-/()^", false);
        let ps = p.parse(&mut input).unwrap();
        print_statesets(&ps);
        println!("=== tree ===");
        println!("{:?}", one_tree(p.g.start(), &ps));
    }
}

#[test]
fn chained_terminals() {
    // E -> X + +  (and other variants)
    // X -> <epsilon>
    let rule_variants = vec![
        vec!["X", "+"],
        vec!["+", "X"],
        vec!["X", "+", "+"],
        vec!["+", "+", "X"],
        vec!["+", "X", "+"],
    ];
    for variant in rule_variants {
        let tokens = match variant.len() {
            2 => "+", 3 => "++", _ => unreachable!()
        };
        let gb = GrammarBuilder::new()
          .symbol("E")
          .symbol("X")
          .symbol(("+", |n: &str| n == "+"))
          .rule("E", &variant)
          .rule("X", &[]);
        let p = EarleyParser::new(gb.into_grammar("E"));
        let mut input = DelimTokenizer::from_str(tokens, "+", false);
        let ps = p.parse(&mut input).unwrap();
        print_statesets(&ps);
        println!("=== tree === variant {:?} === input {}", variant, tokens);
        println!("{:?}", one_tree(p.g.start(), &ps));
    }
}

#[test]
fn natural_lang() {
    let gb = GrammarBuilder::new()
      .symbol(("N", |n: &str| {
        n == "time" || n == "flight" || n == "banana" ||
        n == "flies" || n == "boy" || n == "telescope"
      }))
      .symbol(("D", |n: &str| {
        n == "the" || n == "a" || n == "an"
      }))
      .symbol(("V", |n: &str| {
        n == "book" || n == "eat" || n == "sleep" || n == "saw"
      }))
      .symbol(("P", |n: &str| {
        n == "with" || n == "in" || n == "on" || n == "at" || n == "through"
      }))
      .symbol(("[name]", |n: &str| n == "john" || n == "houston"))
      .symbol("PP")
      .symbol("NP")
      .symbol("VP")
      .symbol("VP")
      .symbol("S")
      .rule("NP", &["D", "N"])
      .rule("NP", &["[name]"])
      .rule("NP", &["NP", "PP"])
      .rule("PP", &["P", "NP"])
      .rule("VP", &["V", "NP"])
      .rule("VP", &["VP", "PP"])
      .rule("S", &["NP", "VP"])
      .rule("S", &["VP"]);
    let p = EarleyParser::new(gb.into_grammar("S"));
    let inputs = vec![
        "book the flight through houston",
        "john saw the boy with the telescope",
    ];
    for input in inputs.iter() {
        println!("============ input: {}", input);
        let mut input = DelimTokenizer::from_str(input, " ", true);
        let ps = p.parse(&mut input).unwrap();
        println!("=== tree ===");
        for t in all_trees(p.g.start(), &ps) { println!("{:?}", t); }
    }
}