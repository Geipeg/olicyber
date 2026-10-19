use anyhow::Result;
use regex::regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

fn main() -> Result<()> {
    let client = Client::builder().cookie_store(true).build()?;

    // 1. Fetch the very first page to start the session
    let mut html_page = client
        .get("http://infinite.challs.olicyber.it/")
        .send()?
        .text()?;

    let mut solved_count = 0;

    loop {
        let document = Html::parse_document(&html_page);
        let test_type_selector =
            Selector::parse("body > center:nth-child(1) > h2:nth-child(1)").unwrap();

        let element = match document.select(&test_type_selector).next() {
            Some(el) => el.html(),
            None => {
                println!("No test type found. Full page:\n{}", html_page);
                break;
            }
        };

        // Prepare the answer body based on the test type
        let mut post_request = client.post("http://infinite.challs.olicyber.it/");

        match element.as_str() {
            "<h2>ART TEST</h2>" => {
                let question_selector =
                    Selector::parse("body > center:nth-child(1) > p:nth-child(3)").unwrap();
                let question = document.select(&question_selector).next().unwrap().html();

                let question_regex = regex!(r#"Premi il pulsante di colore (Rosso|Verde|Blu)\?"#);
                if let Some(caps) = question_regex.captures(&question) {
                    let color = &caps[1];
                    // Depending on how the form is structured, you might need "color=Rosso" or similar.
                    // Let's use standard form body format:
                    post_request = post_request.body(format!("{}={}", color, color));
                }
            }
            "<h2>GRAMMAR TEST</h2>" => {
                let question_selector =
                    Selector::parse("body > center:nth-child(1) > p:nth-child(3)").unwrap();
                let question = document.select(&question_selector).next().unwrap().html();

                let question_regex = regex!(r#"Quante "(\w)" ci sono nella parola "(\w+)"\?"#);
                if let Some(caps) = question_regex.captures(&question) {
                    let character = &caps[1];
                    let word = &caps[2];
                    let count = word.matches(character).count();

                    post_request = post_request.body(format!("letter={}", count));
                }
            }
            "<h2>MATH TEST</h2>" => {
                let question_selector =
                    Selector::parse("body > center:nth-child(1) > p:nth-child(3)").unwrap();
                let question = document.select(&question_selector).next().unwrap().html();

                let question_regex = regex!(r#"Quanto fa (\d+) \+ (\d+)\?"#);
                if let Some(caps) = question_regex.captures(&question) {
                    let a = caps[1].parse::<i32>()?;
                    let b = caps[2].parse::<i32>()?;
                    let sum = a + b;

                    post_request = post_request.body(format!("sum={}", sum));
                }
            }
            other => {
                // If it's not a known test, it's likely the flag page!
                println!("🏁 Finished or reached final page:\n{}", html_page);
                break;
            }
        };

        // 2. Send the POST request and capture the NEXT page from its response!
        let response = post_request.send()?;
        html_page = response.text()?;

        solved_count += 1;
        println!("[{}] Solved a test successfully!", solved_count);
    }

    Ok(())
}
