#![recursion_limit="512"]
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use biscuit_auth::{token::Biscuit, crypto::{KeyPair, PublicKey}, error};
use log::*;
use rand::prelude::*;
use std::default::Default;

struct Model {
    link: ComponentLink<Self>,
    token: Token,
}

enum Kind {
    Fact,
    Rule,
    Caveat,
}

enum Msg {
    AddBlock,
    DeleteBlock { block_index: usize },
    SetBlockEnabled { block_index: usize, enabled: bool },
    AddElement { kind: Kind, block_index: usize, },
    DeleteElement { kind: Kind, block_index: usize, element_index: usize, },
    SetEnabled { enabled: bool, kind: Kind, block_index: usize, element_index: usize, },
    Update { kind: Kind, block_index: usize, element_index: usize, value: String, },

    AddVerifierElement { kind: Kind },
    DeleteVerifierElement { kind: Kind, element_index: usize, },
    SetVerifierEnabled { enabled: bool, kind: Kind, element_index: usize, },
    VerifierUpdate { kind: Kind, element_index: usize, value: String, },

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
            Msg::AddElement { kind, block_index } => {
                let block = if block_index == 0 {
                    &mut self.token.authority
                } else {
                    &mut self.token.blocks[block_index-1]
                };

                match kind {
                    Kind::Fact => block.facts.push(Fact::default()),
                    Kind::Rule => block.rules.push(Rule::default()),
                    Kind::Caveat => block.caveats.push(Caveat::default()),
                }
            },
            Msg::DeleteElement { kind, block_index, element_index, } => {
                let block = if block_index == 0 {
                    &mut self.token.authority
                } else {
                    &mut self.token.blocks[block_index-1]
                };

                match kind {
                    Kind::Fact => {
                        block.facts.remove(element_index);
                    },
                    Kind::Rule => {
                        block.rules.remove(element_index);
                    },
                    Kind::Caveat => {
                        block.caveats.remove(element_index);
                    },
                }
            },
            Msg::SetEnabled { enabled, kind, block_index, element_index, } => {
                let block = if block_index == 0 {
                    &mut self.token.authority
                } else {
                    &mut self.token.blocks[block_index-1]
                };

                match kind {
                    Kind::Fact => block.facts[element_index].enabled = enabled,
                    Kind::Rule => block.rules[element_index].enabled = enabled,
                    Kind::Caveat => block.caveats[element_index].enabled = enabled,
                }
            },
            Msg::Update { kind, block_index, element_index, value, } => {
                let block = if block_index == 0 {
                    &mut self.token.authority
                } else {
                    &mut self.token.blocks[block_index-1]
                };

                match kind {
                    Kind::Fact => block.facts[element_index].data = value,
                    Kind::Rule => block.rules[element_index].data = value,
                    Kind::Caveat => block.caveats[element_index].data = value,
                }
            },
            Msg::AddVerifierElement { kind } => {
                match kind {
                    Kind::Fact => self.token.verifier.facts.push(Fact::default()),
                    Kind::Rule => self.token.verifier.rules.push(Rule::default()),
                    Kind::Caveat => self.token.verifier.caveats.push(Caveat::default()),
                }
            },
            Msg::DeleteVerifierElement { kind, element_index, } => {
                match kind {
                    Kind::Fact => {
                        self.token.verifier.facts.remove(element_index);
                    },
                    Kind::Rule => {
                        self.token.verifier.rules.remove(element_index);
                    },
                    Kind::Caveat => {
                        self.token.verifier.caveats.remove(element_index);
                    },
                }
            },
            Msg::SetVerifierEnabled { enabled, kind, element_index, } => {

                match kind {
                    Kind::Fact => self.token.verifier.facts[element_index].enabled = enabled,
                    Kind::Rule => self.token.verifier.rules[element_index].enabled = enabled,
                    Kind::Caveat => self.token.verifier.caveats[element_index].enabled = enabled,
                }
            },
            Msg::VerifierUpdate { kind, element_index, value, } => {
                match kind {
                    Kind::Fact => self.token.verifier.facts[element_index].data = value,
                    Kind::Rule => self.token.verifier.rules[element_index].data = value,
                    Kind::Caveat => self.token.verifier.caveats[element_index].data = value,
                }
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
            <div id="biscuit-wrapper">
                <h2>{ "Biscuit Token playground" }</h2>
                <div id="token">
                    <em>{"Blocks:"}</em>
                    <ul>
                        { self.view_block(0, &self.token.authority) }
                        { (self.token.blocks.iter()
                            .enumerate())
                            .map(|(id, block)| self.view_block(id+1, block))
                            .collect::<Html>() }
                        <li><button onclick=self.link.callback(move |_| {
                            Msg::AddBlock
                        })>{ "Add Block" }</button></li>
                    </ul>
                    <em>{"Token content:"}</em>
                    <input
                        type="text"
                        size="40"
                        value = { self.token.serialized.as_deref().unwrap_or("") }
                    />
                    <pre id="token-content">
                        { self.token.biscuit.as_ref().map(|b| b.print()).unwrap_or_else(String::new) }
                    </pre>
                </div>
                { self.view_verifier(&self.token.verifier) }
            </div>
        }
    }
}

impl Model {
    fn view_block(&self, block_index: usize, block: &Block) -> Html {
        let is_enabled = block.enabled;

        html! {
            <li class={ if is_enabled { "" } else { "block-disabled" } }>
                <div class="block">
                    <div>

                        <h3>{ if block_index == 0 {
                            "Authority block".to_string()
                        } else {
                            format!("Block {}", block_index)
                        }
                        }</h3>
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
                    </div>

                { "Facts:" }
                <button onclick=self.link.callback(move |_| {
                    Msg::AddElement { kind: Kind::Fact, block_index }
                })>{ "+" }</button>
                <ul>
                    { for block.facts.iter().enumerate()
                        .map(|(fact_index, fact)| self.view_fact(block_index, fact_index, fact))
                    }
                </ul>

                { "Rules:" }
                <button onclick=self.link.callback(move |_| {
                    Msg::AddElement { kind: Kind::Rule, block_index }
                })>{ "+" }</button>
                <ul>
                    { for block.rules.iter().enumerate()
                        .map(|(rule_index, rule)| self.view_rule(block_index, rule_index, rule))
                    }
                </ul>

                { "Caveats:" }
                <button onclick=self.link.callback(move |_| {
                    Msg::AddElement { kind: Kind::Caveat, block_index }
                })>{ "+" }</button>
                <ul>
                    { for block.caveats.iter().enumerate()
                        .map(|(caveat_index, caveat)| self.view_caveat(block_index, caveat_index, caveat))
                    }
                </ul>
                </div>
            </li>
        }
    }

    fn view_fact(&self, block_index: usize, element_index: usize, fact: &Fact) -> Html {
        let is_enabled = fact.enabled;

        html! {
            <li>
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteElement { kind: Kind::Fact, block_index, element_index }
                })>{ "-" }</button>
                <input type="checkbox"
                    onclick = self.link.callback(move |_| {
                        Msg::SetEnabled {kind: Kind::Fact, enabled: !is_enabled, block_index, element_index }
                    })
                    checked = { is_enabled }
                />
                <input
                    type="text"
                    size="40"
                    class= { if fact.parsed { "" } else { "parse_error" } }
                    value = { fact.data.clone() }
                    disabled = if !fact.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::Update { kind: Kind::Fact, block_index, element_index, value: e.value }
                    })
                    />
           </li>
        }
    }

    fn view_rule(&self, block_index: usize, element_index: usize, rule: &Rule) -> Html {
        let is_enabled = rule.enabled;

        html! {
            <li>
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteElement { kind: Kind::Rule, block_index, element_index }
                })>{ "-" }</button>
                <input type="checkbox"
                    onclick = self.link.callback(move |_| {
                        Msg::SetEnabled {kind: Kind::Rule, enabled: !is_enabled, block_index, element_index }
                    })
                    checked = { is_enabled }
                />
                <input
                    type="text"
                    size="40"
                    class= { if rule.parsed { "" } else { "parse_error" } }
                    value = { rule.data.clone() }
                    disabled = if !rule.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::Update { kind: Kind::Rule, block_index, element_index, value: e.value }
                    })
                    />
           </li>
        }
    }

    fn view_caveat(&self, block_index: usize, element_index: usize, caveat: &Caveat) -> Html {
        let is_enabled = caveat.enabled;

        html! {
            <li
                class= { caveat.class() }
            >
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteElement { kind: Kind::Caveat, block_index, element_index }
                })>{ "-" }</button>
                <input type="checkbox"
                    onclick = self.link.callback(move |_| {
                        Msg::SetEnabled {kind: Kind::Caveat, enabled: !is_enabled, block_index, element_index }
                    })
                    checked = { is_enabled }
                />
                <input
                    type="text"
                    size="40"
                    class= { caveat.class() }
                    value = { caveat.data.clone() }
                    disabled = if !caveat.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::Update { kind: Kind::Caveat, block_index, element_index, value: e.value }
                    })
                    />
           </li>
        }
    }

    fn view_verifier(&self, verifier: &Verifier) -> Html {
        html! {
            <div id="verifier">
                <h3>{"Verifier"}</h3>

                { "Facts:" }
                <button onclick=self.link.callback(move |_| {
                    Msg::AddVerifierElement { kind: Kind::Fact }
                })>{ "+" }</button>

                <ul>
                    { for verifier.facts.iter().enumerate()
                        .map(|(fact_index, fact)| self.view_verifier_fact(fact_index, fact))
                    }
                </ul>

                { "Rules:" }
                <button onclick=self.link.callback(move |_| {
                    Msg::AddVerifierElement { kind: Kind::Rule }
                })>{ "+" }</button>

                <ul>
                    { for verifier.rules.iter().enumerate()
                        .map(|(rule_index, rule)| self.view_verifier_rule(rule_index, rule))
                    }
                </ul>

                { "Caveats:" }
                <button onclick=self.link.callback(move |_| {
                    Msg::AddVerifierElement { kind: Kind::Caveat }
                })>{ "+" }</button>

                <ul>
                    { for verifier.caveats.iter().enumerate()
                        .map(|(caveat_index, caveat)| self.view_verifier_caveat(caveat_index, caveat))
                    }
                </ul>

                <h4>{"Output"}</h4>
                <p id="verifier-result">{ match &verifier.error {
                    Some(e) => format!("Error: {:?}", e),
                    None => "Success".to_string(),
                } }</p>

                <pre id="verifier-world">{ &verifier.output }</pre>

            </div>
        }
    }

    fn view_verifier_fact(&self, element_index: usize, fact: &Fact) -> Html {
        let is_enabled = fact.enabled;

        html! {
            <li>
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteVerifierElement { kind: Kind::Fact, element_index }
                })>{ "-" }</button>
                <input type="checkbox"
                    onclick = self.link.callback(move |_| {
                        Msg::SetVerifierEnabled {kind: Kind::Fact, enabled: !is_enabled, element_index }
                    })
                    checked = { is_enabled }
                />
                <input
                    type="text"
                    size="40"
                    class= { if fact.parsed { "" } else { "parse_error" } }
                    value = { fact.data.clone() }
                    disabled = if !fact.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::VerifierUpdate { kind: Kind::Fact, element_index, value: e.value }
                    })
                    />
           </li>
        }
    }

    fn view_verifier_rule(&self, element_index: usize, rule: &Rule) -> Html {
        let is_enabled = rule.enabled;

        html! {
            <li>
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteVerifierElement { kind: Kind::Rule, element_index }
                })>{ "-" }</button>
                <input type="checkbox"
                    onclick = self.link.callback(move |_| {
                        Msg::SetVerifierEnabled {kind: Kind::Rule, enabled: !is_enabled, element_index }
                    })
                    checked = { is_enabled }
                />
                <input
                    type="text"
                    size="40"
                    class= { if rule.parsed { "" } else { "parse_error" } }
                    value = { rule.data.clone() }
                    disabled = if !rule.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::VerifierUpdate { kind: Kind::Rule, element_index, value: e.value }
                    })
                    />
           </li>
        }
    }

    fn view_verifier_caveat(&self, element_index: usize, caveat: &Caveat) -> Html {
        let is_enabled = caveat.enabled;

        html! {
            <li
                class= { caveat.class() }
            >
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteVerifierElement { kind: Kind::Caveat, element_index }
                })>{ "-" }</button>
                <input type="checkbox"
                    onclick = self.link.callback(move |_| {
                        Msg::SetVerifierEnabled {kind: Kind::Caveat, enabled: !is_enabled, element_index }
                    })
                    checked = { is_enabled }
                />
                <input
                    type="text"
                    size="40"
                    class= { caveat.class() }
                    value = { caveat.data.clone() }
                    disabled = if !caveat.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::VerifierUpdate { kind: Kind::Caveat, element_index, value: e.value }
                    })
                    />
           </li>
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
struct Caveat {
    pub data: String,
    pub parsed: bool,
    pub enabled: bool,
    pub succeeded: Option<bool>,
}

impl Default for Caveat {
    fn default() -> Self {
        Caveat {
            data: String::new(),
            parsed: true,
            enabled: true,
            succeeded: None,
        }
    }
}

impl Caveat {
    pub fn new(s: &str) -> Self {
        Caveat {
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
                Some(true) => "caveat_success",
                Some(false) => "caveat_failure",
            }
        }
    }
}

#[derive(Clone,Debug)]
struct Block {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>,
    pub caveats: Vec<Caveat>,
    pub enabled: bool,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            facts: Vec::new(),
            rules: Vec::new(),
            caveats: Vec::new(),
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

        token.authority.facts.push(Fact::new("right(#authority, \"/folder1/file1\", #read)"));
        token.authority.facts.push(Fact::new("right(#authority, \"/folder1/file1\", #write)"));
        token.authority.facts.push(Fact::new("right(#authority, \"/folder2/file1\", #read)"));

        token.blocks[0].caveats.push(Caveat::new("*check(#read) <- operation(#ambient, #read)"));
        token.blocks[1].caveats.push(Caveat::new("*check($file) <- resource(#ambient, $file) @ $file matches /folder1/*"));

        // simulate verification for PUT /blog1/article1
        token.verifier.facts.push(Fact::new("resource(#ambient, \"/folder1/file1\")"));
        token.verifier.facts.push(Fact::new("operation(#ambient, #read)"));
        token.verifier.caveats.push(Caveat::new("*check($file) <- resource(#ambient, $file), operation(#ambient, $op), right(#authority, $file, $op)"));

        token
    }

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

        // add the rights verification caveat
        token.verifier.caveats.push(Caveat::new(
            "*verified($blog_id, $article_id, $operation) <-
             blog(#ambient, $blog_id),
             article(#ambient, $blog_id, $article_id),
             operation(#ambient, $operation),
             right(#authority, $blog_id, $article_id, $operation)",
             ));

        token
    }

    fn generate(&mut self) {
        info!("generate token: {:?}", self);
        let mut rng: StdRng = SeedableRng::seed_from_u64(0);
        let root = KeyPair::new(&mut rng);

        let mut builder = Biscuit::builder(&root);

        for fact in self.authority.facts.iter_mut() {
            if fact.enabled {
                fact.parsed = builder.add_authority_fact(fact.data.as_str()).is_ok();
            }
        }

        for rule in self.authority.rules.iter_mut() {
            if rule.enabled {
                rule.parsed = builder.add_authority_rule(rule.data.as_str()).is_ok();
            }
        }

        for caveat in self.authority.caveats.iter_mut() {
            if caveat.enabled {
                caveat.parsed = builder.add_authority_caveat(caveat.data.as_str()).is_ok();
            }
        }

        let mut token = builder.build(&mut rng).unwrap();

        for block in self.blocks.iter_mut() {
            if block.enabled {
                let temp_keypair = KeyPair::new(&mut rng);
                let mut builder = token.create_block();

                for fact in block.facts.iter_mut() {
                    if fact.enabled {
                        fact.parsed = builder.add_fact(fact.data.as_str()).is_ok();
                    }
                }

                for rule in block.rules.iter_mut() {
                    if rule.enabled {
                        rule.parsed = builder.add_rule(rule.data.as_str()).is_ok();
                    }
                }

                for caveat in block.caveats.iter_mut() {
                    if caveat.enabled {
                        caveat.parsed = builder.add_caveat(caveat.data.as_str()).is_ok();
                    }
                    caveat.succeeded = Some(true);
                }

                token = token.append(&mut rng, &temp_keypair, builder).unwrap();
            }
        }

        self.verifier.verify(&token, root.public());
        let v = token.to_vec().unwrap();
        self.serialized = Some(base64::encode_config(&v[..], base64::URL_SAFE));
        self.biscuit = Some(token);

        if let Some(error::Token::FailedLogic(error::Logic::FailedCaveats(v))) = self.verifier.error.as_ref() {
            for e in v.iter() {
                match e {
                    error::FailedCaveat::Verifier(error::FailedVerifierCaveat { caveat_id, .. }) => {
                        self.verifier.caveats[*caveat_id as usize].succeeded = Some(false);
                    },
                    error::FailedCaveat::Block(error::FailedBlockCaveat { block_id, caveat_id, .. }) => {
                        if *block_id == 0 {
                            self.authority.caveats[*caveat_id as usize].succeeded = Some(false);
                        } else {
                            self.blocks[*block_id as usize - 1].caveats[*caveat_id as usize].succeeded = Some(false);
                        }
                    },
                }

            }
        }
    }
}

#[derive(Clone,Debug,Default)]
struct Verifier {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>,
    pub caveats: Vec<Caveat>,
    pub error: Option<error::Token>,
    pub output: String,
}

impl Verifier {
    pub fn verify(&mut self, token: &Biscuit, root: PublicKey) {
        self.error = None;

        let mut verifier = token.verify(root).unwrap();

        for fact in self.facts.iter_mut() {
            if fact.enabled {
                fact.parsed = verifier.add_fact(fact.data.as_str()).is_ok();
            }
        }

        for rule in self.rules.iter_mut() {
            if rule.enabled {
                rule.parsed = verifier.add_rule(rule.data.as_str()).is_ok();
            }
        }

        for caveat in self.caveats.iter_mut() {
            if caveat.enabled {
                caveat.parsed = verifier.add_caveat(caveat.data.as_str()).is_ok();
            }
            caveat.succeeded = Some(true);
        }

        if let Err(e) = verifier.verify() {
            self.error = Some(e);
        }

        self.output = verifier.print_world();
    }
}
