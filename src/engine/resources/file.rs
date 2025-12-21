use std::error::Error;

// TODO: fix url base on live server and normal web server
#[allow(unused)]
#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        "http://127.0.0.1:5500",
        "day23_xr_wgpu/xr_wgpu" // location.origin().unwrap(),
                                // option_env!("OUT_DIR").unwrap_or("res")
    ))
    .unwrap();
    base.join(file_name).unwrap()
}



#[allow(unused)]
#[cfg(target_arch = "wasm32")]
pub async fn load_string(file_name: &str) -> Result<String, Box<dyn Error + 'static>>
{
    let url = format_url(file_name);
    let txt = reqwest::get(url).await?.text().await?;
    Ok(txt)
}
    
#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
pub fn load_string(file_name: &str) -> Result<String, Box<dyn Error + 'static>>
{
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;
    Ok(txt)
}



#[allow(unused)]
#[cfg(target_arch = "wasm32")]
pub async fn load_binary(file_name: &str) -> Result<Vec<u8>, Box<dyn Error + 'static>>
{
    let url = format_url(file_name);
    let data = reqwest::get(url).await?.bytes().await?.to_vec();
    Ok(data)
}

#[allow(unused)]
#[cfg(not(target_arch = "wasm32"))]
pub fn load_binary(file_name: &str) -> Result<Vec<u8>, Box<dyn Error + 'static>>
{
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;
    Ok(data)
}
