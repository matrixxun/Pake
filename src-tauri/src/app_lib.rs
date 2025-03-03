use tauri::{Manager, Window};
use tauri::Emitter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct QuadUrls {
    url1: String,
    url2: String,
    url3: String,
    url4: String,
}

#[tauri::command]
fn load_urls(window: Window, urls: QuadUrls) {
    window.emit("load-urls", urls).unwrap();
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // 获取主窗口
            let main_window = app.get_webview_window("main");
            
            if let Some(window) = main_window {
                println!("初始化主窗口");
                
                // 设置 webview 允许加载 iframe
                window.eval(r#"
                    console.log("初始化主界面");
                    // 移除所有 iframe
                    document.querySelectorAll('iframe').forEach(iframe => {
                        iframe.parentNode.removeChild(iframe);
                    });
                    
                    // 创建四个区域
                    const container = document.createElement('div');
                    container.style.display = 'grid';
                    container.style.gridTemplateColumns = '1fr 1fr';
                    container.style.gridTemplateRows = '1fr 1fr';
                    container.style.gap = '4px';
                    container.style.height = '100vh';
                    container.style.background = '#f0f0f0';
                    
                    // 创建四个区域
                    const quadrants = [];
                    for (let i = 0; i < 4; i++) {
                        const quadrant = document.createElement('div');
                        quadrant.className = 'quadrant';
                        quadrant.id = `quadrant${i+1}`;
                        quadrant.style.display = 'flex';
                        quadrant.style.alignItems = 'center';
                        quadrant.style.justifyContent = 'center';
                        quadrant.style.background = 'white';
                        quadrant.style.borderRadius = '8px';
                        quadrant.style.position = 'relative';
                        
                        const loading = document.createElement('div');
                        loading.className = 'loading';
                        loading.textContent = '加载中...';
                        loading.style.fontSize = '18px';
                        
                        quadrant.appendChild(loading);
                        container.appendChild(quadrant);
                        quadrants.push(quadrant);
                    }
                    
                    // 添加到页面
                    document.body.innerHTML = '';
                    document.body.style.margin = '0';
                    document.body.style.padding = '0';
                    document.body.style.height = '100vh';
                    document.body.style.overflow = 'hidden';
                    document.body.appendChild(container);
                    
                    // 存储区域位置信息，供后续使用
                    window.quadrantPositions = quadrants.map(q => {
                        const rect = q.getBoundingClientRect();
                        return {
                            id: q.id,
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height
                        };
                    });
                    
                    // 通知Rust获取位置信息
                    setTimeout(() => {
                        window.__TAURI__.invoke('get_quadrant_positions', { 
                            positions: window.quadrantPositions 
                        });
                    }, 500);
                "#).unwrap_or_else(|e| eprintln!("执行 JS 失败: {}", e));
            } else {
                eprintln!("找不到主窗口");
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_urls,
            get_quadrant_positions,
            create_embedded_webviews
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize, Clone)]
struct QuadrantPosition {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[tauri::command]
fn get_quadrant_positions(app_handle: tauri::AppHandle, positions: Vec<QuadrantPosition>) {
    // 获取位置信息后创建嵌入式webview
    create_embedded_webviews(app_handle, positions).unwrap();
}

#[tauri::command]
fn create_embedded_webviews(app_handle: tauri::AppHandle, positions: Vec<QuadrantPosition>) -> Result<(), String> {
    let urls = vec![
        "https://claude.ai/new",
        "https://chat.openai.com",
        "https://grok.x.ai",
        "https://chat.deepseek.com"
    ];
    
    let titles = vec![
        "Claude AI",
        "ChatGPT",
        "Grok",
        "DeepSeek"
    ];
    
    // 获取主窗口
    let main_window = app_handle.get_webview_window("main")
        .ok_or_else(|| "找不到主窗口".to_string())?;
    
    // 为每个区域创建一个嵌入式webview
    for (i, position) in positions.iter().enumerate() {
        if i >= urls.len() {
            break;
        }
        
        let webview_id = format!("webview{}", i+1);
        
        // 创建webview并嵌入到主窗口中
        let webview = tauri::WebviewWindowBuilder::new(
            &app_handle,
            &webview_id,
            tauri::WebviewUrl::External(urls[i].parse().unwrap())
        )
        .title(titles[i])
        .inner_size(position.width, position.height)
        .position(position.x, position.y)
        .parent_window(main_window.label())
        .visible(true)
        .decorations(false) // 无边框
        .always_on_top(true)
        .build();
        
        if let Err(e) = webview {
            eprintln!("创建 {} 窗口失败: {}", titles[i], e);
        } else {
            println!("成功创建 {} 窗口", titles[i]);
        }
    }
    
    Ok(())
}