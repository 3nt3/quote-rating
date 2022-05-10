use wasm_bindgen::JsCast;
use web_sys::EventTarget;
use web_sys::HtmlInputElement;
use yew::events::*;
use yew::prelude::*;

enum Msg {
    SetValue(String),
}

struct Model {
    value: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            value: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetValue(new_value) => {
                self.value = new_value
                    .chars()
                    .filter(|x| x.to_owned() != '#')
                    .collect::<String>();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();

        let on_input = link.batch_callback(|e: InputEvent| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.map(|input| Msg::SetValue(input.value()))
        });

        html! {
            <div>
              <input type="text" oninput={on_input} value={self.value.clone()} />
              <p>{ self.value.to_uppercase() }</p>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
