#![recursion_limit="512"]
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use biscuit_auth::{token::Biscuit, crypto::{KeyPair, PublicKey}, error};
use log::*;
use rand::prelude::*;
use std::default::Default;

struct Model {
    link: ComponentLink<Self>,
    value: i64,
    token: Token,
}

enum Kind {
    Fact,
    Rule,
    Caveat,
}

enum Msg {
    AddOne,
    AddBlock,
    DeleteBlock { block_index: usize },
    SetBlockEnabled { block_index: usize, enabled: bool },
    AddElement { kind: Kind, block_index: usize, },
    DeleteElement { kind: Kind, block_index: usize, element_index: usize, },
    SetEnabled { enabled: bool, kind: Kind, block_index: usize, element_index: usize, },
    Update { kind: Kind, block_index: usize, element_index: usize, value: String, },
    None,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut token = Token::default();
        token.authority.facts.push(Fact {
            data: "user(#authority, \"user_1234\")".to_string(),
            parsed: false,
            enabled: true,
        });
        token.authority.facts.push(Fact {
            data: "hello".to_string(),
            parsed: false,
            enabled: true,
        });
        token.generate();
        Self { link, value: 0, token }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value += 1,
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
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
                <h2>{ "Biscuit Token" }</h2>
                <ul>
                    { self.view_block(0, &self.token.authority) }
                    { (self.token.blocks.iter()
                        .enumerate())
                        .map(|(id, block)| self.view_block(id+1, block))
                        .collect::<Html>() }
                    <li><button onclick=self.link.callback(move |_| {
                        Msg::AddBlock
                    })>{ "+" }</button></li>
                </ul>
                <pre>
                    { self.token.biscuit.as_ref().map(|b| b.print()).unwrap_or_else(String::new) }
                </pre>
            </div>
        }
    }
}

impl Model {
    fn view_block(&self, block_index: usize, block: &Block) -> Html {
        let is_enabled = block.enabled;

        html! {
            <li>
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteBlock { block_index }
                })>{ "-" }</button>
                <button onclick=self.link.callback(move |_| {
                    Msg::SetBlockEnabled {enabled: !is_enabled, block_index }
                })>{ "/" }</button>
                <h3>{ if block_index == 0 {
                    "authority".to_string()
                } else {
                    format!("Block {}", block_index)
                }
                }</h3>
            { "Facts:" }
                <ul>
                    { for block.facts.iter().enumerate()
                        .map(|(fact_index, fact)| self.view_fact(block_index, fact_index, fact))
                    }
                    <li><button onclick=self.link.callback(move |_| {
                        Msg::AddElement { kind: Kind::Fact, block_index }
                    })>{ "+" }</button></li>
                </ul>
            { "Rules:" }
                <ul>
                    { for block.rules.iter().enumerate()
                        .map(|(rule_index, rule)| self.view_rule(block_index, rule_index, rule))
                    }
                    <li><button onclick=self.link.callback(move |_| {
                        Msg::AddElement { kind: Kind::Rule, block_index }
                    })>{ "+" }</button></li>
                </ul>
            { "Caveats:" }
                <ul>
                    { for block.caveats.iter().enumerate()
                        .map(|(caveat_index, caveat)| self.view_caveat(block_index, caveat_index, caveat))
                    }
                    <li><button onclick=self.link.callback(move |_| {
                        Msg::AddElement { kind: Kind::Caveat, block_index }
                    })>{ "+" }</button></li>
                </ul>
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
                <button onclick=self.link.callback(move |_| {
                    Msg::SetEnabled { kind: Kind::Fact, enabled: !is_enabled, block_index, element_index }
                })>{ "/" }</button>
                <input
                    type="text"
                    size="50"
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
                <button onclick=self.link.callback(move |_| {
                    Msg::SetEnabled { kind: Kind::Rule, enabled: !is_enabled, block_index, element_index }
                })>{ "/" }</button>
                <input
                    type="text"
                    size="50"
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
            <li>
                <button onclick=self.link.callback(move |_| {
                    Msg::DeleteElement { kind: Kind::Caveat, block_index, element_index }
                })>{ "-" }</button>
                <button onclick=self.link.callback(move |_| {
                    Msg::SetEnabled { kind: Kind::Caveat, enabled: !is_enabled, block_index, element_index }
                })>{ "/" }</button>
                <input
                    type="text"
                    size="50"
                    class= { if caveat.parsed { "" } else { "parse_error" } }
                    value = { caveat.data.clone() }
                    disabled = if !caveat.enabled { true } else { false }

                    oninput=self.link.callback(move |e: InputData| {
                        Msg::Update { kind: Kind::Caveat, block_index, element_index, value: e.value }
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
}

impl Token {
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
                }

                token = token.append(&mut rng, &temp_keypair, builder).unwrap();
            }
        }

        self.biscuit = Some(token);
    }

}
