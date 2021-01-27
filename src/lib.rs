#![recursion_limit="512"]
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use biscuit_auth::{token::Biscuit, crypto::{KeyPair, PublicKey}, error,
  parser::{parse_source, SourceResult},
};
use log::*;
use rand::prelude::*;
use std::default::Default;
use nom::Offset;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Model {
    link: ComponentLink<Self>,
    token: Token,
}

enum Msg {
    AddBlock,
    DeleteBlock { block_index: usize },
    SetBlockEnabled { block_index: usize, enabled: bool },
    UpdateBlockCode { block_index: usize, value: String, },
    UpdateVerifierCode { value: String, },

    None,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut token = Token::files_scenario();
        token.generate();
        Self { link, token }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddBlock => {
                self.token.blocks.push(Block::default());
            },
            Msg::DeleteBlock { block_index } => {
                self.token.blocks.remove(block_index - 1);
            },
            Msg::SetBlockEnabled { block_index, enabled } => {
                if block_index == 0 {
                    self.token.authority.enabled = enabled;
                } else {
                    self.token.blocks[block_index - 1].enabled = enabled;
                }
            },
            Msg::UpdateBlockCode { block_index, value, } => {
                let block = if block_index == 0 {
                    &mut self.token.authority
                } else {
                    &mut self.token.blocks[block_index-1]
                };

                block.code = value;
            },
            Msg::UpdateVerifierCode { value, } => {
                self.token.verifier.code = value;
            },
            Msg::None => {},
        }

        self.token.generate();

        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        info!("will show view");

        html! {
            <div>
                <div id="header">
                    <a href="https://www.clever-cloud.com/"><img src="/logo.svg" /></a>
                    <h2>
                    { "Biscuit Token playground" }
                    </h2>
                </div>
                <div id="biscuit-wrapper">
                    <div id="explain">
                        <p>{"This is a live demo of the "}
                          <a href="https://github.com/CleverCloud/biscuit">{"Biscuit authentication and authorization tokens"}</a>{", where you can test different authorization policies. Each token is made of blocks, each block represents one attenuation level: you can restrict the rights of a token by adding a new block. Authorization policies are written in Datalog, where facts represent data, rules generate more facts from existing facts, and checks validate the presence of some facts. To pass the verification phase, all of the checks must succeed."}
                        </p>
                        <p>{"Test the behaviour of the example token by activating or deactivating blocks or their data, changing conditions (like "}<em>{"#read"}</em>{" operation to "}<em>{"#write"}</em>{" and see how the verifier will react"}</p>
                    </div>

                    <div id="token" class="container">
                        <h3>{"Token"}</h3>
                        <ul id="block-list">
                            { self.view_block(0, &self.token.authority) }
                            { (self.token.blocks.iter()
                                .enumerate())
                                .map(|(id, block)| self.view_block(id+1, block))
                                .collect::<Html>() }
                            <li><button onclick=self.link.callback(move |_| {
                                Msg::AddBlock
                            })>{ "Add Block" }</button></li>
                        </ul>
                        <div class="sub-container">
                            <em>{"Token content "}</em>
                            <input
                                type="text"
                                size="45"
                                value = { self.token.serialized.as_deref().unwrap_or("") }
                            />
                            <pre id="token-content">
                                { self.token.biscuit.as_ref().map(|b| b.print()).unwrap_or_else(String::new) }
                            </pre>
                        </div>
                    </div>
                    { self.view_verifier(&self.token.verifier) }
                </div>
            </div>
        }
    }
}

impl Model {
    fn view_block(&self, block_index: usize, block: &Block) -> Html {
        let is_enabled = block.enabled;

        html! {
            <li class={ if is_enabled { "sub-container" } else { "sub-container block-disabled" } }>
                <div class="block">
                    <div>

                        <span>
                            <button onclick=self.link.callback(move |_| {
                                Msg::DeleteBlock { block_index }
                            })
                                hidden = { block_index == 0 }
                            >{ "-" }</button>
                            <input type="checkbox"
                                onclick = self.link.callback(move |_| {
                                    Msg::SetBlockEnabled {enabled: !is_enabled, block_index }
                                })
                                checked = { is_enabled }
                                hidden = { block_index == 0 }
                            />
                            <h4 style="display:inline">{ if block_index == 0 {
                                "Authority block".to_string()
                            } else {
                                format!("Block {}", block_index)
                            }
                            }</h4>
                        </span>
                        <br />
                    </div>

                    <textarea
                        class="code-buffer"
                        style="display: none"
                        id={ format!("block-code-{}-buffer", block_index) }
                        oninput=self.link.callback(move |e: InputData| {
                            Msg::UpdateBlockCode { block_index, value: e.value }
                        })
                    >{ &block.code }</textarea>
                    <textarea
                        class="code"
                        id={ format!("block-code-{}", block_index) }
                    ></textarea>
                </div>
            </li>
        }
    }

    fn view_verifier(&self, verifier: &Verifier) -> Html {
        html! {
            <div id="verifier" class="container">
                <h3>{"Verifier"}</h3>

                <div class="sub-container">
                    <textarea
                        class="code-buffer"
                        id = "verifier-code-buffer"
                        style="display: none"
                        oninput=self.link.callback(move |e: InputData| {
                            Msg::UpdateVerifierCode { value: e.value }
                        })
                    >{ &verifier.code }</textarea>
                    <textarea
                        class="code"
                        id = "verifier-code"
                    ></textarea>
                </div>

                <div class="sub-container">
                    <h4>{"Verifier result"}</h4>
                    <p id="verifier-result">{ match &verifier.error {
                        Some(e) => format!("Error: {:?}", e),
                        None => "Success".to_string(),
                    } }</p>

                    <pre id="verifier-world">{ &verifier.output }</pre>
                </div>

            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));


    App::<Model>::new().mount_to_body();
}

#[derive(Clone,Debug)]
struct Fact {
    pub data: String,
    pub parsed: bool,
    pub enabled: bool,
}

impl Default for Fact {
    fn default() -> Self {
        Fact {
            data: String::new(),
            parsed: true,
            enabled: true,
        }
    }
}

impl Fact {
    pub fn new(s: &str) -> Self {
        Fact {
            data: s.to_string(),
            parsed: true,
            enabled: true,
        }
    }
}

#[derive(Clone,Debug)]
struct Rule {
    pub data: String,
    pub parsed: bool,
    pub enabled: bool,
}

impl Default for Rule {
    fn default() -> Self {
        Rule {
            data: String::new(),
            parsed: true,
            enabled: true,
        }
    }
}

impl Rule {
    pub fn new(s: &str) -> Self {
        Rule {
            data: s.to_string(),
            parsed: true,
            enabled: true,
        }
    }
}

#[derive(Clone,Debug)]
struct Check {
    pub data: String,
    pub parsed: bool,
    pub enabled: bool,
    pub succeeded: Option<bool>,
}

impl Default for Check {
    fn default() -> Self {
        Check {
            data: String::new(),
            parsed: true,
            enabled: true,
            succeeded: None,
        }
    }
}

impl Check {
    pub fn new(s: &str) -> Self {
        Check {
            data: s.to_string(),
            parsed: true,
            enabled: true,
            succeeded: None,
        }
    }

    pub fn class(&self) -> &str {
        if !self.parsed {
            "parse_error"
        } else {
            match self.succeeded {
                None => "",
                Some(true) => "check_success",
                Some(false) => "check_failure",
            }
        }
    }
}

#[derive(Clone,Debug)]
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
    let line_start = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

    // Find the line that includes the subslice:
    // ind the *last* newline before the substring starts
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
    let column_start = line.offset(span) + 1;

    let offset = offset + span.len();
    let prefix = &input.as_bytes()[..offset];

    // Count the number of newlines in the first `offset` bytes of input
    let line_end = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

    // Find the line that includes the subslice:
    // ind the *last* newline before the substring starts
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

    SourcePosition { line_start, column_start, line_end, column_end }
}


#[derive(Clone,Debug)]
struct Block {
    pub code: String,
    pub checks: Vec<SourcePosition>,
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

#[derive(Clone,Debug,Default)]
struct Token {
    pub authority: Block,
    pub blocks: Vec<Block>,
    pub biscuit: Option<Biscuit>,
    pub serialized: Option<String>,
    pub verifier: Verifier,
}

impl Token {
    fn files_scenario() -> Self {
        let mut token = Token::default();
        token.blocks.push(Block::default());
        token.blocks.push(Block::default());

        token.authority.code = r#"right(#authority, "/folder1/file1", #read)
right(#authority, "/folder1/file1", #write)
right(#authority, "/folder2/file1", #read)

check if operation(#ambient, #read)
"#.to_string();

        token.blocks[0].code = r#"check if resource(#ambient, $file), $file.starts_with("/folder1/")
"#.to_string();


        // simulate verification for PUT /blog1/article1
        token.verifier.code = r#"resource(#ambient, "/folder1/file1")
operation(#ambient, #read)

check if resource(#ambient, $file), operation(#ambient, $op), right(#authority, $file, $op)

allow if true
"#.to_string();

        /*
        token.verifier.facts.push(Fact::new("resource(#ambient, \"/folder1/file1\")"));
        token.verifier.facts.push(Fact::new("operation(#ambient, #read)"));
        token.verifier.checks.push(Check::new("*check($file) <- resource(#ambient, $file), operation(#ambient, $op), right(#authority, $file, $op)"));
        */

        token
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

    fn generate(&mut self) {
        //info!("generate token: {:?}", self);
        info!("will generate token");

        unsafe { clear_marks() };

        let mut rng: StdRng = SeedableRng::seed_from_u64(0);
        let root = KeyPair::new_with_rng(&mut rng);

        let mut builder = Biscuit::builder(&root);

        self.authority.checks.clear();
        if let Ok((_, authority_parsed)) = parse_source(&self.authority.code) {
            for (_,fact) in authority_parsed.facts.iter() {
                builder.add_authority_fact(fact.clone()).unwrap();
            }

            for (_,rule) in authority_parsed.rules.iter() {
                builder.add_authority_rule(rule.clone()).unwrap();
            }

            for (i,check) in authority_parsed.checks.iter() {
                builder.add_authority_check(check.clone()).unwrap();
                let position = get_position(&self.authority.code, i);
                self.authority.checks.push(position);
            }
        }

        let mut token = builder.build_with_rng(&mut rng).unwrap();

        for block in self.blocks.iter_mut() {

            block.checks.clear();

            if block.enabled {
                let temp_keypair = KeyPair::new_with_rng(&mut rng);
                let mut builder = token.create_block();

                if let Ok((_, block_parsed)) = parse_source(&block.code) {
                    for (_,fact) in block_parsed.facts.iter() {
                        builder.add_fact(fact.clone()).unwrap();
                    }

                    for (_,rule) in block_parsed.rules.iter() {
                        builder.add_rule(rule.clone()).unwrap();
                    }

                    for (i,check) in block_parsed.checks.iter() {
                        builder.add_check(check.clone()).unwrap();
                        let position = get_position(&block.code, i);
                        block.checks.push(position);
                    }
                }

                token = token.append_with_rng(&mut rng, &temp_keypair, builder).unwrap();

            } else {
                /*
                for check in block.checks.iter_mut() {
                    check.succeeded = None;
                }
                */
            }
        }

        self.verifier.verify(&token, root.public());
        let v = token.to_vec().unwrap();
        self.serialized = Some(base64::encode_config(&v[..], base64::URL_SAFE));
        self.biscuit = Some(token);

        if let Some(error::Token::FailedLogic(error::Logic::FailedChecks(v))) = self.verifier.error.as_ref() {
            for e in v.iter() {
                match e {
                    error::FailedCheck::Verifier(error::FailedVerifierCheck { check_id, .. }) => {
                        //self.verifier.checks[*check_id as usize].succeeded = Some(false);
                        let position = &self.verifier.checks[*check_id as usize];
                        info!("will update verifier marks for {}: {:?}", check_id, position);
                        unsafe { mark(
                          "verifier-code",
                          position.line_start,
                          position.column_start,
                          position.line_end,
                          position.column_end,
                          "background: #c1f1c1;"
                        )};
                    },
                    error::FailedCheck::Block(error::FailedBlockCheck { block_id, check_id, .. }) => {
                        let block = if *block_id == 0 {
                            &self.authority
                        } else {
                            &self.blocks[*block_id as usize - 1]
                        };
                        let position = &block.checks[*check_id as usize];
                        info!("will update block[{}] marks for {}: {:?}", block_id, check_id, position);
                        unsafe { mark(
                          &format!("block-code-{}", block_id),
                          position.line_start,
                          position.column_start,
                          position.line_end,
                          position.column_end,
                          "background: #c1f1c1;"
                        )};
                    },
                }

            }
        }
    }
}

#[derive(Clone,Debug,Default)]
struct Verifier {
    pub code: String,
    /*
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>,
    pub checks: Vec<Check>,
    */
    pub checks: Vec<SourcePosition>,
    pub error: Option<error::Token>,
    pub output: String,
}

impl Verifier {
    pub fn verify(&mut self, token: &Biscuit, root: PublicKey) {
        self.error = None;

        let mut verifier = token.verify(root).unwrap();

        info!("verifier source:\n{}", self.code);

        let res = parse_source(&self.code);
        if let Err(e) = res {
            self.error = Some(error::Token::ParseError);
            self.output = e.to_string();
            return;
        }

        self.checks.clear();

        let (_, parsed) = parse_source(&self.code).unwrap();

        for (_,fact) in parsed.facts.iter() {
            verifier.add_fact(fact.clone()).unwrap();
        }

        for (_,rule) in parsed.rules.iter() {
            verifier.add_rule(rule.clone()).unwrap();
        }

        for (i,check) in parsed.checks.iter() {
            verifier.add_check(check.clone()).unwrap();
            let position = get_position(&self.code, i);
            self.checks.push(position);
        }

        for (_,policy) in parsed.policies.iter() {
            verifier.add_policy(policy.clone()).unwrap();
        }

        if let Err(e) = verifier.verify() {
            self.error = Some(e);
        }

        self.output = verifier.print_world();
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

#[wasm_bindgen(inline_js = "export function mark(id, line_start, column_start, line_end, column_end, style) {
    console.log(\"adding mark in \"+id);

    var mark = window.editors[id].markText(
      {line: line_start, ch: column_start},
      {line: line_end, ch: column_end},
      {css: style}
    );
    window.marks.push(mark);
    console.log(window.marks);
}")]
extern "C" {
    fn mark(id:&str, line_start: usize, column_start: usize, line_end: usize, column_end: usize, css: &str);
}
