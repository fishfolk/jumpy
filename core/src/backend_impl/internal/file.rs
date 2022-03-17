use std::fs;
use std::path::Path;
use cfg_if::cfg_if;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

use crate::file::Error;

#[cfg(target_arch = "wasm32")]
async fn load_file_wasm<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&path, &opts)?;

    //request.headers().set("Accept", "application/json")?;

    let fetch = web_sys::window().unwrap().fetch_with_request(&request);
    let resp_value = JsFuture::from(fetch).await?;
    let response: Response = resp_value.dyn_into().unwrap();
    let buffer = JsFuture::from(response.text()?).await?.unwrap();

    Ok(buffer)
}

#[cfg(target_os = "android")]
async fn load_file_android<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    unimplemented!("File loading for android is not implemented yet")
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn load_file_sync<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    match fs::read(&path) {
        Err(err) => Err(Error::new(path, err)),
        Ok(res) => Ok(res),
    }
}

pub async fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    #[cfg(target_arch = "wasm32")]
    let res = load_file_wasm(path).await?;
    #[cfg(target_os = "android")]
    let res = load_file_android(path).await?;
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    let res = load_file_sync(path)?;

    Ok(res)
}