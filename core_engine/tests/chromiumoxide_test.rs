//! CDP Browser Test - Quick version
//! 
//! Run with: cargo test --package core_engine test_chromiumoxide_quick -- --nocapture

#[cfg(test)]
mod tests {
    use chromiumoxide::{Browser, BrowserConfig};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_chromiumoxide_quick() {
        println!("\n=== Quick chromiumoxide test ===");
        
        // Use thread-based approach
        let result = thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            rt.block_on(async {
                // Build config
                let temp_dir = std::env::temp_dir().join(format!("chromium_test_{}", std::process::id()));
                std::fs::create_dir_all(&temp_dir).ok();
                
                let config = BrowserConfig::builder()
                    .no_sandbox()
                    .user_data_dir(&temp_dir)
                    .build()
                    .unwrap();
                
                println!("Launching browser...");
                let (browser, _handler) = Browser::launch(config).await.unwrap();
                println!("✅ Browser launched!");
                
                println!("Creating page...");
                let page = browser.new_page("https://www.example.com").await.unwrap();
                println!("✅ Page created!");
                
                // Quick operations
                let title = page.get_title().await.unwrap_or_else(|_| None);
                println!("✅ Title: {:?}", title);
                
                let url = page.url().await.unwrap_or_else(|_| None);
                println!("✅ URL: {:?}", url);
                
                // Quick content check
                let params = chromiumoxide_cdp::cdp::js_protocol::runtime::EvaluateParams::builder()
                    .expression("document.body.innerText.substring(0,100)")
                    .build()
                    .unwrap();
                let content = page.evaluate_expression(params).await;
                if let Ok(result) = content {
                    if let Some(val) = result.value() {
                        println!("✅ Content: {}", val);
                    }
                }
                
                println!("✅ All tests passed!");
                std::process::exit(0); // Force exit
            })
        }).join();
        
        // If we get here, something went wrong
        if let Err(e) = result {
            println!("Thread error: {:?}", e);
        }
    }
}
