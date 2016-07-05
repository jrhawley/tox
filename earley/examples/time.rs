extern crate linenoise;
extern crate regex;
extern crate lexers;
extern crate toxearley as earley;
extern crate time;

use std::iter::FromIterator;
use std::collections::HashSet;
use earley::Subtree;
use regex::Regex;

fn day_of_week(d: &str) -> bool {
    let days = HashSet::<&str>::from_iter(vec![
        "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"
    ]);
    days.contains(d)
}

fn month(m: &str) -> bool {
    let months = HashSet::<&str>::from_iter(vec![
        "january", "february", "march", "april", "may", "june",
        "july", "august", "september", "october", "november", "december"
    ]);
    months.contains(m)
}

fn ordinals(n: &str) -> bool {
    let ord = HashSet::<&str>::from_iter(vec![
        "first", "second", "third", "fourth", "fifth", "sixth",
    ]);
    ord.contains(n)
}

fn ordinal_digits(n: &str) -> bool {
    let ord = Regex::new(r"\d+ ?(st|nd|rd|th)").unwrap();
    ord.is_match(n)
}

// https://github.com/wit-ai/duckling/blob/master/resources/languages/en/rules/time.clj
fn build_grammar() -> earley::Grammar {
    let mut gb = earley::GrammarBuilder::new();
    gb.symbol(("named-day", day_of_week))
      .symbol(("ordinal (digits)", ordinal_digits))
      .symbol(("ordinal (names)", ordinals))
      .symbol(("ordinal", |n: &str| ordinals(n) || ordinal_digits(n)))
      .symbol(("month", month))
      ;

    gb.symbol(("this|next", |n: &str| n == "this" || n == "next"))
      .symbol(("the", |n: &str| n == "the"))
      ;

    gb.symbol("this|next <day-of-week>")
      .symbol("the <day-of-month> (ordinal)")
      .symbol("<named-month> <day-of-month> (ordinal)")
      ;

    gb.rule("this|next <day-of-week>", &["this|next", "named-day"])    // next tuesday
      .rule("the <day-of-month> (ordinal)", &["the", "ordinal"])       // the 2nd
      .rule("<named-month> <day-of-month> (ordinal)", &["month", "ordinal"])
      ;

    // tie rules to start symbol
    gb.rule("S", &["this|next <day-of-week>"])
      .rule("S", &["the <day-of-month> (ordinal)"])
      .rule("S", &["<named-month> <day-of-month> (ordinal)"])
      ;

    gb.into_grammar("S")
}

struct TimeRange {
    start: time::Tm,
    end: time::Tm,
}

//impl TimeRange {
    //fn today() -> TimeRange {
    //}
//}

struct TimeContext;

impl TimeContext {
    fn eval(&mut self, n: &Subtree) -> Option<TimeRange> {
        None
    }
}

//trait TimeMod {
    //fn apply(ctx: &Context, n: &Subtree);
//}

//struct NCycle();

//impl TimeMode for NCycle {
    //fn apply(ctx: &Context, n: &Subtree) {
    //}
//}

fn semantics() {
    //""
}



fn dotprinter(node: &Subtree, n: usize) {
    match node {
        &Subtree::Node(ref term, ref value) => println!("  \"{}. {}\" -> \"{}. {}\"", n, term, n + 1, value),
        &Subtree::SubT(ref spec, ref childs) => for (nn, c) in childs.iter().enumerate() {
            let x = match c {
                &Subtree::Node(ref term, _) => term,
                &Subtree::SubT(ref sspec, _) => sspec,
            };
            println!("  \"{}. {}\" -> \"{}. {}\"", n, spec, n + nn + 100, x);
            dotprinter(&c, n + nn + 100);
        }
    }
}

fn main() {
    let parser = earley::EarleyParser::new(build_grammar());

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).
            collect::<Vec<String>>().join(" ");
        match parser.parse(&mut lexers::DelimTokenizer::from_str(&input, " ", true)) {
            Ok(estate) => {
                let tree = earley::one_tree(parser.g.start(), &estate);
                println!("digraph x {{");
                dotprinter(&tree, 0);
                println!("}}");
            },
            Err(e) => println!("Parse err: {:?}", e)
        }
        return;
    }

    //let mut ctx = HashMap::new();
    //while let Some(input) = linenoise::input("~> ") {
        //linenoise::history_add(&input[..]);
        //match parser.parse(&mut lexers::DelimTokenizer::from_str(&input, " ", true)) {
            //Ok(estate) => {
                //let tree = earley::one_tree(parser.g.start(), &estate);
                //let val = xeval(&tree, &mut ctx)[0];
                //println!("{:?}", val);
            //},
            //Err(e) => println!("Parse err: {:?}", e)
        //}
    //}
}
