#![recursion_limit = "512"]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, HtmlInputElement};
//use yew::prelude::*;
use biscuit_auth::{
    crypto::{KeyPair, PublicKey},
    error,
    parser::{parse_source, SourceResult},
    token::Biscuit,
    token::builder,
    token::verifier::{Verifier, VerifierLimits},
};
use log::*;
use nom::Offset;
use rand::prelude::*;
use std::default::Default;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn testBiscuit(parent_selector: &str) {
    unsafe {
        log("testBiscuit");
    }
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    //let collection = document.get_elements_by_class_name("code");
    let collection = document.query_selector_all(&format!("{} .block-code", parent_selector)).unwrap();

    unsafe { clear_marks() };

    let mut block_codes = Vec::new();
    for i in 0..collection.length() {
        let element = collection.item(i).unwrap();
        let textarea = element.dyn_ref::<HtmlTextAreaElement>().unwrap();
        unsafe {
            //log(&format!("got content: {}", textarea.value()));
        }
        block_codes.push(textarea.value());
    }

    info!("will generate token");

    let mut rng: StdRng = SeedableRng::seed_from_u64(0);
    let root = KeyPair::new_with_rng(&mut rng);

    let mut builder = Biscuit::builder(&root);

    let mut authority = Block::default();
    let mut blocks = Vec::new();

    let mut token_opt = None;

    if !block_codes.is_empty() {
        set_token_content(parent_selector, String::new());

        match parse_source(&block_codes[0]) {
            Err(e) => {
                error!("error: {:?}", e);
                let selector = format!("{} .block-code-0", parent_selector);
                set_parse_errors(&selector, &block_codes[0], e);
            },
            Ok((_, authority_parsed)) => {
                for (_, fact) in authority_parsed.facts.iter() {
                    builder.add_authority_fact(fact.clone()).unwrap();
                }

                for (_, rule) in authority_parsed.rules.iter() {
                    builder.add_authority_rule(rule.clone()).unwrap();
                }

                for (i, check) in authority_parsed.checks.iter() {
                    builder.add_authority_check(check.clone()).unwrap();
                    let position = get_position(&block_codes[0], i);
                    authority.checks.push((position, true));
                }
            }
        }

        let mut token = builder.build_with_rng(&mut rng).unwrap();

        for (i, code) in (&block_codes[1..]).iter().enumerate() {
            let mut block = Block::default();

            let temp_keypair = KeyPair::new_with_rng(&mut rng);
            let mut builder = token.create_block();

            match parse_source(&code) {
                Err(e) => {
                    error!("error: {:?}", e);
                    let selector = format!("{} .block-code-{}", parent_selector, i+1);
                    set_parse_errors(&selector, &code, e);
                },
                Ok((_, block_parsed)) => {
                    for (_, fact) in block_parsed.facts.iter() {
                        builder.add_fact(fact.clone()).unwrap();
                    }

                    for (_, rule) in block_parsed.rules.iter() {
                        builder.add_rule(rule.clone()).unwrap();
                    }

                    for (i, check) in block_parsed.checks.iter() {
                        builder.add_check(check.clone()).unwrap();
                        let position = get_position(&code, i);
                        block.checks.push((position, true));
                    }
                }
            }

            token = token
                .append_with_rng(&mut rng, &temp_keypair, builder)
                .unwrap();

            blocks.push(block);
        }

        let v = token.to_vec().unwrap();
        //self.serialized = Some(base64::encode_config(&v[..], base64::URL_SAFE));
        //self.biscuit = Some(token);
        set_token_content(parent_selector, token.print());

        token_opt = Some(token);
    }

    if let Some(verifier_element) = document.query_selector(&format!("{} .verifier-code", parent_selector)).unwrap() {
        let textarea = verifier_element.dyn_ref::<HtmlTextAreaElement>().unwrap();
        unsafe {
            //log(&format!("got content: {}", textarea.value()));
        }

        let verifier_code = textarea.value();

        set_verifier_result(parent_selector, String::new(), String::new());

        let mut verifier = match token_opt {
            Some(token) => token.verify(root.public()).unwrap(),
            None => Verifier::new().unwrap(),
        };

        //info!("verifier source:\n{}", &verifier_code);

        let verifier_result;
        let output;

        let res = parse_source(&verifier_code);
        if let Err(e) = res {
            verifier_result = Err(error::Token::ParseError);
            output = format!("errors: {:?}", e);
            error!("error: {:?}", e);
            set_parse_errors(&format!("{} .verifier-code", parent_selector), &verifier_code, e);
        } else {
            let mut verifier_checks = Vec::new();
            let mut verifier_policies = Vec::new();

            let (_, parsed) = res.unwrap();

            for (_, fact) in parsed.facts.iter() {
                verifier.add_fact(fact.clone()).unwrap();
            }

            for (_, rule) in parsed.rules.iter() {
                verifier.add_rule(rule.clone()).unwrap();
            }

            for (i, check) in parsed.checks.iter() {
                verifier.add_check(check.clone()).unwrap();
                let position = get_position(&verifier_code, i);
                // checks are marked as success until they fail
                verifier_checks.push((position, true));
            }

            for (i, policy) in parsed.policies.iter() {
                verifier.add_policy(policy.clone()).unwrap();
                let position = get_position(&verifier_code, i);
                // checks are marked as success until they fail
                verifier_policies.push(position);
            }

            let mut limits = VerifierLimits::default();
            limits.max_time = std::time::Duration::from_secs(2);
            verifier_result = verifier.verify_with_limits(limits);

            output = verifier.print_world();

            match &verifier_result {
                Err(error::Token::FailedLogic(error::Logic::FailedChecks(v))) => {
                    for e in v.iter() {
                        match e {
                            error::FailedCheck::Verifier(error::FailedVerifierCheck {
                                check_id, ..
                            }) => {

                                verifier_checks[*check_id as usize].1 = false;
                            }
                            error::FailedCheck::Block(error::FailedBlockCheck {
                                block_id,
                                check_id,
                                ..
                            }) => {
                                let block = if *block_id == 0 {
                                    &mut authority
                                } else {
                                    &mut blocks[*block_id as usize - 1]
                                };
                                block.checks[*check_id as usize].1 = false;
                            }
                        }
                    }
                },
                Err(error::Token::FailedLogic(error::Logic::Deny(index))) => {
                    let position = &verifier_policies[*index];
                    unsafe {
                        mark(
                            &format!("{} .verifier-code", parent_selector),
                            position.line_start,
                            position.column_start,
                            position.line_end,
                            position.column_end,
                            "background: #ffa2a2;"
                        )
                    };
                },
                Ok(index) => {
                    let position = &verifier_policies[*index];
                    unsafe {
                        mark(
                            &format!("{} .verifier-code", parent_selector),
                            position.line_start,
                            position.column_start,
                            position.line_end,
                            position.column_end,
                            "background: #c1f1c1;"
                        )
                    };
                },
                _ => {},
            }

            for (position, result) in authority.checks.iter() {
                unsafe {
                    mark(
                        &format!("{} .block-code-0", parent_selector),
                        position.line_start,
                        position.column_start,
                        position.line_end,
                        position.column_end,
                        if *result {
                          "background: #c1f1c1;"
                        } else {
                          "background: #ffa2a2;"
                        },
                    )
                };
            }

            for (id, block) in blocks.iter().enumerate() {
                for (position, result) in block.checks.iter() {
                    unsafe {
                        mark(
                            &format!("{} .block-code-{}", parent_selector, id+1),
                            position.line_start,
                            position.column_start,
                            position.line_end,
                            position.column_end,
                            if *result {
                                "background: #c1f1c1;"
                            } else {
                                "background: #ffa2a2;"
                            },
                        )
                    };
                }
            }

            for (position, result) in verifier_checks.iter() {
                unsafe {
                    mark(
                        &format!("{} .verifier-code", parent_selector),
                        position.line_start,
                        position.column_start,
                        position.line_end,
                        position.column_end,
                        if *result {
                          "background: #c1f1c1;"
                        } else {
                          "background: #ffa2a2;"
                        },
                    )
                };
            }

            if let Some(query_element) = document.query_selector(&format!("{} .query", parent_selector)).unwrap() {
                let input = query_element.dyn_ref::<HtmlInputElement>().unwrap();
                let query = input.value();
                log(&format!("got query content: {}", query));
                set_query_result(parent_selector, String::new());

                if !query.is_empty() {
                    let query_result: Result<Vec<builder::Fact>, biscuit_auth::error::Token> =
                        verifier.query(query.as_str());
                    match query_result {
                        Err(e) => set_query_result(parent_selector, format!("Error: {:?}", e)),
                        Ok(facts) => {
                            let facts: Vec<String> = facts.iter().map(|f| f.to_string()).collect();
                            let result = facts.join(",\n");
                            set_query_result(parent_selector, result);
                        }
                    }
                }
            }
        }

        set_verifier_result(
            parent_selector,
            match &verifier_result {
                Err(e) => format!("Error: {:?}", e),
                Ok(_) => "Success".to_string(),
            },
            output,
        );
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    unsafe { log("wasm run_app") }
}

#[derive(Clone, Debug)]
struct SourcePosition {
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
}

// based on nom's convert_error
fn get_position(input: &str, span: &str) -> SourcePosition {
    let offset = input.offset(span);
    let prefix = &input.as_bytes()[..offset];

    // Count the number of newlines in the first `offset` bytes of input
    let line_start = prefix.iter().filter(|&&b| b == b'\n').count();

    // Find the line that includes the subslice:
    // find the *last* newline before the substring starts
    let line_begin = prefix
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map(|pos| offset - pos)
        .unwrap_or(0);

    // Find the full line after that newline
    let line = input[line_begin..]
        .lines()
        .next()
        .unwrap_or(&input[line_begin..])
        .trim_end();

    // The (1-indexed) column number is the offset of our substring into that line
    let column_start = line.offset(span);

    let offset = offset + span.len();
    let prefix = &input.as_bytes()[..offset];

    // Count the number of newlines in the first `offset` bytes of input
    let line_end = prefix.iter().filter(|&&b| b == b'\n').count();

    // Find the line that includes the subslice:
    // find the *last* newline before the substring starts
    let line_begin = prefix
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map(|pos| offset - pos)
        .unwrap_or(0);

    // Find the full line after that newline
    let line = input[line_begin..]
        .lines()
        .next()
        .unwrap_or(&input[line_begin..])
        .trim_end();

    // The (1-indexed) column number is the offset of our substring into that line
    let column_end = line.offset(&span[span.len()..]) + 1;

    SourcePosition {
        line_start,
        column_start,
        line_end,
        column_end,
    }
}

#[derive(Clone, Debug)]
struct Block {
    pub code: String,
    pub checks: Vec<(SourcePosition, bool)>,
    pub enabled: bool,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            code: String::new(),
            checks: Vec::new(),
            enabled: true,
        }
    }
}

/*
fn newspaper_scenario() -> Self {
    let mut token = Token::default();
    token.authority.facts.push(Fact::new("user(#authority, \"user_1234\")"));

    // simulate verification for PUT /blog1/article1
    token.verifier.facts.push(Fact::new("blog(#ambient, \"blog1\")"));
    token.verifier.facts.push(Fact::new("article(#ambient, \"blog1\", \"article1\")"));
    token.verifier.facts.push(Fact::new("operation(#ambient, #update)"));

    // add ownership information
    // we only ned to load facts related to the blog and article we're accessing
    token.verifier.facts.push(Fact::new("owner(#authority, \"user_1234\", \"blog1\")"));
    //verifier.add_fact("owner(#authority, \"user_5678\", \"blog2\")")?;
    //verifier.add_fact("owner(#authority, \"user_1234\", \"blog3\")")?;

    // add our authorization policies
    // the owner has all rights
    token.verifier.rules.push(Rule::new(
        "*right(#authority, $blog_id, $article_id, $operation) <-
         article(#ambient, $blog_id, $article_id),
         operation(#ambient, $operation),
         user(#authority, $user_id),
         owner(#authority, $user_id, $blog_id))",
         ));

    // articles can be marked as publicly readable
    token.verifier.rules.push(Rule::new(
        "*right(#authority, $blog_id, $article_id, #read) <-
         article(#ambient, $blog_id, $article_id),
         readable(#authority, $blog_id, $article_id))",
        ));
    // premium users can access some restricted articles
    token.verifier.rules.push(Rule::new(
        "*right(#authority, $blog_id, $article_id, #read) <-
         article(#ambient, $blog_id, $article_id),
         premium_readable(#authority, $blog_id, $article_id),
         user(#authority, $user_id),
         premium_user(#authority, $user_id, $blog_id))",
     ));

    // define teams and roles
    token.verifier.rules.push(Rule::new(
        "*right(#authority, $blog_id, $article_id, $operation) <-
         article(#ambient, $blog_id, $article_id),
         operation(#ambient, $operation),
         user(#authority, $user_id),
         member(#authority, $usr_id, $team_id),
         team_role(#authority, $team_id, $blog_id, #contributor)
         @ $operation in [#read, #write])",
         ));

    // add the rights verification check
    token.verifier.checks.push(Check::new(
        "*verified($blog_id, $article_id, $operation) <-
         blog(#ambient, $blog_id),
         article(#ambient, $blog_id, $article_id),
         operation(#ambient, $operation),
         right(#authority, $blog_id, $article_id, $operation)",
         ));

    token
}*/

fn set_parse_errors(selector: &str, input: &str, errors: Vec<biscuit_auth::parser::Error>) {
    clear_parse_errors(selector);
    error!("got errors: {:?}", errors);
    for e in errors.iter() {
        let position = get_position(input, e.input);
        let message = e.message.as_ref().cloned().unwrap_or_else(|| format!("error: {:?}", e.code));

        error!("position for error({:?}) \"{}\": {:?}", e.code, message, position);

        register_parse_error(selector, message,
                             position.line_start, position.column_start,
                             position.line_end, position.column_end);
    }
}

#[wasm_bindgen(inline_js = "export function clear_marks() {
    for(var i=0; i < window.marks.length; i=i+1) {
        console.log(\"clearing mark \"+i);
        window.marks[i].clear();
    }
    window.marks = [];
}")]
extern "C" {
    fn clear_marks();
}

#[wasm_bindgen(
    inline_js = "export function mark(selector, line_start, column_start, line_end, column_end, style) {
    console.log(\"adding mark in \"+selector);

    var mark = window.editors[selector].markText(
      {line: line_start, ch: column_start},
      {line: line_end, ch: column_end},
      {css: style}
    );
    window.marks.push(mark);
    console.log(window.marks);
}"
)]
extern "C" {
    fn mark(
        selector: &str,
        line_start: usize,
        column_start: usize,
        line_end: usize,
        column_end: usize,
        css: &str,
    );
}

#[wasm_bindgen(inline_js = "export function set_verifier_result(parent, error, world) {
    var element = document.querySelector(parent + \" .verifier-result\");
    element.innerText = error;
    var element = document.querySelector(parent + \" .verifier-world\");
    element.innerText = world;
}")]
extern "C" {
    fn set_verifier_result(parent: &str, error: String, world: String);
}

#[wasm_bindgen(inline_js = "export function set_token_content(parent, content) {
    var element = document.querySelector(parent+\" .token-content\");
    element.innerText = content;
}")]
extern "C" {
    fn set_token_content(parent: &str, content: String);
}

#[wasm_bindgen(inline_js = "export function set_query_result(parent, result) {
    var element = document.querySelector(parent + \" .query-result\");
    element.innerText = result;
}")]
extern "C" {
    fn set_query_result(parent: &str, result: String);
}

#[wasm_bindgen(
    inline_js = "export function register_parse_error(selector, message, line_start, column_start, line_end, column_end) {
    window.editor_lints[selector].push({
        message: message,
        severity: \"error\",
        from: CodeMirror.Pos(line_start, column_start),
        to: CodeMirror.Pos(line_end, column_end),
    });

}")]
extern "C" {
    fn register_parse_error(
        selector: &str,
        message: String,
        line_start: usize,
        column_start: usize,
        line_end: usize,
        column_end: usize);
}

#[wasm_bindgen(
    inline_js = "export function clear_parse_errors(selector) {
    window.editor_lints[selector] = [];
}")]
extern "C" {
    fn clear_parse_errors(selector: &str);
}

