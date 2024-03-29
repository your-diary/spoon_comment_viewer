use std::process;
use std::time::Duration;

use log::{error, info};
use thirtyfour_sync::{error::WebDriverError, prelude::*};

pub struct Selenium {
    driver: WebDriver,
}

impl Selenium {
    pub fn new(
        webdriver_port: usize,
        implicit_timeout: Duration,
        should_maximize_window: bool,
    ) -> Self {
        let mut firefox = DesiredCapabilities::firefox();

        //disables desktop notification
        firefox
            .add_firefox_option(
                "prefs",
                serde_json::json!({"permissions.default.desktop-notification": 1}),
            )
            .unwrap();

        //allows microphone access
        firefox
            .add_firefox_option(
                "prefs",
                serde_json::json!({"permissions.default.microphone": 1}),
            )
            .unwrap();

        let driver = match WebDriver::new(
            format!("http://localhost:{}", webdriver_port).as_str(),
            &firefox,
        ) {
            Ok(o) => o,
            Err(_) => {
                error!("Is `geckodriver` running?");
                process::exit(1);
            }
        };
        driver.set_implicit_wait_timeout(implicit_timeout).unwrap();
        if (should_maximize_window) {
            driver.maximize_window().unwrap();
        }

        Selenium { driver }
    }

    pub fn driver(&self) -> &WebDriver {
        &self.driver
    }

    pub fn close(&self) -> Result<(), WebDriverError> {
        self.driver.close()
    }

    pub fn get(&self, url: &str) -> Result<(), WebDriverError> {
        self.driver.get(url)
    }

    pub fn query(&self, css_selector: &str) -> Result<WebElement, WebDriverError> {
        self.driver.find_element(By::Css(css_selector))
    }

    pub fn query_all(&self, css_selector: &str) -> Result<Vec<WebElement>, WebDriverError> {
        self.driver.find_elements(By::Css(css_selector))
    }

    pub fn click(&self, css_selector: &str) -> Result<(), WebDriverError> {
        self.query(css_selector).and_then(|e| e.click())
    }

    pub fn input(&self, css_selector: &str, s: &str) -> Result<(), WebDriverError> {
        self.query(css_selector).and_then(|e| e.send_keys(s))
    }

    //synchronously calls JavaScript
    //Note the script `script` shall contain a `return` statement to return the evaluated value.
    pub fn execute_javascript(
        &self,
        script: &str,
    ) -> Result<serde_json::value::Value, WebDriverError> {
        Ok(self.driver.execute_script(script)?.value().clone())
    }

    pub fn inner_text(&self, css_selector: &str) -> Result<String, WebDriverError> {
        self.query(css_selector).and_then(|e| e.text())
    }

    pub fn switch_tab(&self, index: usize) -> Result<(), WebDriverError> {
        self.driver
            .switch_to()
            .window(&(self.driver.window_handles().unwrap()[index]))
    }
}

impl Drop for Selenium {
    fn drop(&mut self) {
        info!("Closing the driver...");
    }
}
