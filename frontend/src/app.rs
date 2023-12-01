use common::{backend::BackendRun, weather, WidgetEnum, WidgetId};
use yew::prelude::*;
use yew_router::prelude::*;

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,

    #[at("/hello-server")]
    HelloServer,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <h1>{ "Hello Frontend" }</h1> },
        Route::HelloServer => html! { <HelloServer /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn view_card(title: &'static str, img_url: Option<&'static str>, content: Html) -> Html {
    html! {
        <div class="w-96 h-48 rounded bg-green-400 text-white p-6">
            {for img_url.map(|url| html! {
                <img class="float-right w-12" src={url} alt="Logo" />
            })}
            <h1 class="text-4xl mb-8">{title}</h1>
            {content}
        </div>
    }
}

#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);

    // Request `/api/widgets` once
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let result = fetch_api::<Vec<WidgetEnum>>("widgets").await;
                    data.set(Some(result));
                });
            }

            || {}
        });
    }

    let widgets = match data.as_ref() {
        None => {
            html! {
                <div class="text-red">{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                {
                    // construct the right component for each widget
                    data.iter().map(|widget| {
                        match widget {
                            WidgetEnum::Weather(w) => html!{<WeatherWidget definition={w.clone()} />},
                        }
                    }).collect::<Html>()
                }
            }
        }
        Some(Err(err)) => {
            html! {
                <div class="text-red">{"Error loading widgets: "}{err}</div>
            }
        }
    };

    html! {
        <div class="flex flex-col h-screen">
        <nav class="bg-green-400 h-16 px-8 py-2">
            <div class="container flex mx-auto gap-6 items-center h-full">
                <h1 class="font-bold text-2xl text-white">{"Dashboard"}</h1>
                <div class="flex-1"></div>
                // {for links.iter().map(|(label, href)| html! {
                //     <a class=link_classes href={*href}>{label}</a>
                // })}
            </div>
        </nav>
        <div class="bg-gray-50 flex-1 flex py-16 px-8 items-center gap-6 justify-center">
            { widgets }
        </div>
    </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct WeatherWidgetProps {
    definition: common::weather::Widget,
}

#[function_component(WeatherWidget)]
fn weather_widget(props: &WeatherWidgetProps) -> Html {
    let WeatherWidgetProps { definition } = props;
    // get the most recent run here,
    // TODO: create a nicer looking UI etc

    let state = use_state(|| None);

    // Request `/api/widgets` once
    {
        let state = state.clone();
        let id = definition.id.clone();
        use_effect(move || {
            if state.is_none() {
                spawn_local(async move {
                    let result = fetch_widget_state::<weather::Output>(id, "latest").await;
                    state.set(Some(result));
                });
            }

            || {}
        });
    }

    let content = match state.as_ref() {
        None => {
            html! {
                <div>{"No server response"}</div>
            }
        }
        Some(Ok(data)) => {
            html! {
                <div class="widgets">
                {
                    format!("{:?}", data)
                }
                </div>
            }
        }
        Some(Err(err)) => {
            html! {
                <div>{"Error requesting data from server: "}{err}</div>
            }
        }
    };

    html! {
     {   view_card("Weather", None, content)}
    }
}

async fn fetch_api<T: serde::de::DeserializeOwned>(endpoint: &str) -> Result<T, String> {
    let resp = Request::get(&format!("/api/{}", endpoint))
        .send()
        .await
        .unwrap();

    if !resp.ok() {
        Err(format!(
            "Error fetching data {} ({})",
            resp.status(),
            resp.status_text()
        ))
    } else {
        // successful, get the text and try to parse it into the list of widgets
        resp.text()
            .await
            .map_err(|err| err.to_string())
            .and_then(|text| {
                serde_json::from_str::<T>(&text).map_err(|err| format!("{} content: {}", err, text))
            })
    }
}

async fn fetch_widget_state<O: serde::de::DeserializeOwned>(
    id: WidgetId,
    version: &str,
) -> Result<Option<Result<O, String>>, String> {
    let result = fetch_api::<BackendRun>(&format!("widget/{}/{}", id, version)).await;

    result.and_then(|run| {
        run.result
            .map_err(|err| format!("Backend Error: {:?}", err))
            .map(|r| {
                r.map(|text| {
                    serde_json::from_str::<O>(&text)
                        .map_err(|err| format!("{:?} content: {}", err, text))
                })
            })
    })
}
