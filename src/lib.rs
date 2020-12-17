#![recursion_limit="256"]
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use biscuit_auth::{token::Biscuit, crypto::KeyPair, error};
use log::*;
use rand::prelude::*;

/*
struct Model {
    link: ComponentLink<Self>,
    value: i64,
}

enum Msg {
    AddOne,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, value: 0 }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value += 1,
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}
*/

struct Model {
    link: ComponentLink<Self>,
    value: i64,
    token: Token,
}

enum Msg {
    AddOne,
    AddFact { index: usize },
    AddRule { index: usize },
    AddCaveat { index: usize },
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut token = Token::default();
        token.authority.facts.push(Fact {
            data: "user(#authority, \"user_1234\")".to_string(),
            parsed: false,
        });
        token.authority.facts.push(Fact {
            data: "hello".to_string(),
            parsed: false,
        });
        token.generate();
        Self { link, value: 0, token }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value += 1,
            Msg::AddFact { index } => {
                if index == 0 {
                    self.token.authority.facts.push(Fact::default());
                } else {
                    self.token.blocks[index-1].facts.push(Fact::default());
                }
            },
            Msg::AddRule { index } => {
                if index == 0 {
                    self.token.authority.rules.push(Rule::default());
                } else {
                    self.token.blocks[index-1].rules.push(Rule::default());
                }
            },
            Msg::AddCaveat { index } => {
                if index == 0 {
                    self.token.authority.caveats.push(Caveat::default());
                } else {
                    self.token.blocks[index-1].caveats.push(Caveat::default());
                }
            },
        }
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
                </ul>
                <pre>
                    { self.token.biscuit.as_ref().map(|b| b.print()).unwrap_or_else(String::new) }
                </pre>
            </div>
        }
    }
}

impl Model {
    fn view_block(&self, id: usize, block: &Block) -> Html {
        html! {
            <li>
            { "Facts:" }
                <ul>
                    { for block.facts.iter().map(|fact| self.view_fact(fact)) }
                    <button onclick=self.link.callback(move |_| Msg::AddFact { index: id })>{ "+" }</button>
                </ul>
            { "Rules:" }
                <ul>
                    { for block.rules.iter().map(|rule| self.view_rule(rule)) }
                    <button onclick=self.link.callback(move |_| Msg::AddRule { index: id })>{ "+" }</button>
                </ul>
            { "Caveats:" }
                <ul>
                    { for block.caveats.iter().map(|caveat| self.view_caveat(caveat)) }
                    <button onclick=self.link.callback(move |_| Msg::AddCaveat { index: id })>{ "+" }</button>
                </ul>
            </li>
        }
    }

    fn view_fact(&self, fact: &Fact) -> Html {
        html! {
            <li>
                <input
                    type="text"
                    size="50"
                    class= { if fact.parsed { "" } else { "parse_error" } }
                    value = { fact.data.clone() } />
           </li>
        }
    }

    fn view_rule(&self, rule: &Rule) -> Html {
        html! {
            <li>
                <input
                    type="text"
                    size="50"
                    class= { if rule.parsed { "" } else { "parse_error" } }
                    value = { rule.data.clone() } />
           </li>
        }
    }

    fn view_caveat(&self, caveat: &Caveat) -> Html {
        html! {
            <li>
                <input
                    type="text"
                    size="50"
                    class= { if caveat.parsed { "" } else { "parse_error" } }
                    value = { caveat.data.clone() } />
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

#[derive(Clone,Debug,Default)]
struct Fact {
    pub data: String,
    pub parsed: bool,
}

#[derive(Clone,Debug,Default)]
struct Rule {
    pub data: String,
    pub parsed: bool,
}

#[derive(Clone,Debug,Default)]
struct Caveat {
    pub data: String,
    pub parsed: bool,
    pub succeeded: Option<bool>,
}

#[derive(Clone,Debug,Default)]
struct Block {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>,
    pub caveats: Vec<Caveat>,
}

#[derive(Clone,Debug,Default)]
struct Token {
    pub authority: Block,
    pub blocks: Vec<Block>,
    pub biscuit: Option<Biscuit>,
}

impl Token {
    fn generate(&mut self) {
        info!("generate: {}", line!());
        let mut rng: StdRng = SeedableRng::seed_from_u64(0);
        info!("generate: {}", line!());
        let root = KeyPair::new(&mut rng);
        info!("generate: {}", line!());

        let mut builder = Biscuit::builder(&root);
        info!("generate: {}", line!());

        for fact in self.authority.facts.iter_mut() {
            fact.parsed = builder.add_authority_fact(fact.data.as_str()).is_ok();
        }

        for rule in self.authority.rules.iter_mut() {
            rule.parsed = builder.add_authority_rule(rule.data.as_str()).is_ok();
        }

        for caveat in self.authority.caveats.iter_mut() {
            caveat.parsed = builder.add_authority_caveat(caveat.data.as_str()).is_ok();
        }
        info!("generate: {}", line!());

        let mut token = builder.build(&mut rng).unwrap();
        info!("generate: {}", line!());

        for block in self.blocks.iter_mut() {
            let temp_keypair = KeyPair::new(&mut rng);
        info!("generate: {}", line!());
            let mut builder = token.create_block();
        info!("generate: {}", line!());

            for fact in block.facts.iter_mut() {
                fact.parsed = builder.add_fact(fact.data.as_str()).is_ok();
            }

            for rule in block.rules.iter_mut() {
                rule.parsed = builder.add_rule(rule.data.as_str()).is_ok();
            }

            for caveat in block.caveats.iter_mut() {
                caveat.parsed = builder.add_caveat(caveat.data.as_str()).is_ok();
            }

        info!("generate: {}", line!());
            token = token.append(&mut rng, &temp_keypair, builder).unwrap();
        info!("generate: {}", line!());
        }

        info!("generate: {}", line!());
        self.biscuit = Some(token);
        info!("generate: {}", line!());
    }

}
