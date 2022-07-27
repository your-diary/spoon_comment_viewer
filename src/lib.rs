pub mod comment;
pub mod config;
pub mod selenium;

use thirtyfour_sync::{error::WebDriverError, WebDriverCommands};

use selenium::Selenium;

// use config::Config;

pub fn login(z: &Selenium, twitter_id: &str, twitter_password: &str) -> Result<(), WebDriverError> {
    z.driver().get("https://www.spooncast.net/jp/")?;

    z.click(".btn-login")?;
    z.click(".btn-twitter button")?;

    z.switch_tab(1)?;

    z.input("#username_or_email", twitter_id)?;
    z.input("#password", twitter_password)?;
    z.click("#allow")?;

    z.switch_tab(0)?;

    Ok(())
}

pub fn comment(z: &Selenium, s: &str) -> Result<(), WebDriverError> {
    z.input("textarea", s)?;
    z.click("button[title='送信']")?;
    Ok(())
}
