use dioxus::prelude::*;
use serde::Deserialize;

//Mit LLM generierter Newsfeed mit Artikeln von dev.to die #Rust als Tag haben, um leeren Platz im Dashboard vorübergehend zu füllen
#[component]
pub fn DashboardNewsWidget() -> Element {
    let news_resource = use_resource(|| async move {
        let url = "https://dev.to/api/articles?tag=rust&per_page=3";
        let client = match reqwest::Client::builder()
            .user_agent("DioxusDashboard/1.0")
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                println!("DevTo Error: Client konnte nicht gebaut werden: {}", e);
                return None;
            }
        };
        let response = match client.get(url).send().await {
            Ok(res) => {
                if !res.status().is_success() {
                    println!("DevTo Error: API antwortet mit Status {}", res.status());
                    return None;
                }
                res
            }
            Err(e) => {
                println!("DevTo Error: Netzwerkfehler beim Senden: {}", e);
                return None;
            }
        };

        let articles: Vec<DevToArticle> = match response.json().await {
            Ok(json) => json,
            Err(e) => {
                println!("DevTo Error: Konnte JSON nicht parsen: {}", e);
                return None;
            }
        };

        Some(articles)
    });

    rsx! {
        div {
            style: "background: #171923; border: 1px solid rgba(255,255,255,0.1); border-radius: 12px; padding: 20px; flex: 1; color: white; display: flex; flex-direction: column; overflow: hidden;",
            div {
                style: "margin-bottom: 16px;",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { "🦀" }
                        span { style: "font-weight: 600; font-size: 14px; text-transform: uppercase; letter-spacing: 1px; color: #9ca3af;", "Dev.to Rust News" }
                    }
                    span {
                        style: "font-size: 10px; font-weight: 700; color: #D34516; background: #d3451639; padding: 3px 8px; border-radius: 6px;",
                        "#rust"
                    }
                }
                div {
                    style: "font-size: 11px; color: #666e7a; margin-left: 28px; margin-top: 2px; font-style: italic;",
                    "Articles including the #rust tag"
                }
            }
            div {
                style: "display: flex; flex-direction: column; gap: 12px; overflow-y: auto; flex: 1; padding-right: 4px;",

                match &*news_resource.read() {
                    Some(Some(articles)) => rsx! {
                        for article in articles {
                            a {
                                href: "{article.url}",
                                target: "_blank",
                                style: "display: flex; flex-direction: column; gap: 4px; padding: 12px; background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05); border-radius: 8px; text-decoration: none; transition: background 0.2s;",
                                class: "hover:bg-white/5",

                                span { style: "color: white; font-size: 13px; font-weight: 500; line-height: 1.4;", "{article.title}" }
                                span { style: "color: #3A6BFF; font-size: 11px;", "{article.readable_publish_date}" }
                            }
                        }
                    },
                    Some(None) => rsx! { span { style: "color: #EF4444; font-size: 12px;", "Fehler beim Laden (siehe Terminal)." } },
                    None => rsx! { span { style: "color: #9ca3af; font-size: 12px;", "Lade News..." } }
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct DevToArticle {
    pub title: String,
    pub url: String,
    pub readable_publish_date: String,
}
