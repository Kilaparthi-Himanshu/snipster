
use anyhow::{bail, Context, Result};

use windows::{
    core::HSTRING,
    Foundation::Uri,
    ApplicationModel::DataTransfer::SharedStorageAccessManager,
    Storage::StorageDeleteOption,
    System::Launcher,
    Win32::System::WinRT::{
        RoInitialize,
        RoUninitialize,
        RO_INIT_MULTITHREADED,
    },
};

use url::Url;

const CAPTURE_URI: &str =
    "ms-screenclip://capture/image?rectangle&user-agent=Snipster&redirect-uri=snipster://capture-response&api-version=1.2";

pub fn launch_capture() -> Result<()> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;

        let result = launch_capture_internal();

        RoUninitialize();

        result
    }
}

fn launch_capture_internal() -> Result<()> {
    let uri = Uri::CreateUri(&HSTRING::from(CAPTURE_URI))?;

    let operation = Launcher::LaunchUriAsync(&uri)?;
    let launched = operation.join()?;

    if !launched {
        bail!("Windows could not launch Snipping Tool.");
    }

    println!("Snipping Tool launch request accepted.");

    Ok(())
}

pub fn handle_callback_uri(callback_uri: &str) -> Result<()> {
    let uri = Url::parse(callback_uri)
        .context("Invalid Snipster callback URI")?;

    if uri.scheme() != "snipster"
        || uri.host_str() != Some("capture-response")
    {
        bail!("Unexpected protocol activation.");
    }

    let mut code = None;
    let mut token = None;
    let mut reason = None;

    for (key, value) in uri.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "file-access-token" => token = Some(value.into_owned()),
            "reason" => reason = Some(value.into_owned()),
            _ => {}
        }
    }

    let code = code.context("Callback did not contain a status code")?;

    if code != "200" {
        println!(
            "Capture did not complete. Code: {}, reason: {}",
            code,
            reason.unwrap_or_else(|| "Unknown".to_string())
        );

        return Ok(());
    }

    let token = token.context(
        "Successful capture did not contain a file-access-token"
    )?;

    println!("Capture completed. Redeeming image token...");

    redeem_and_copy(&token)?;

    println!("Screenshot copied to clipboard!");

    Ok(())
}

fn redeem_and_copy(token: &str) -> Result<()> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;

        let result = redeem_and_copy_internal(token);

        RoUninitialize();

        result
    }
}

fn redeem_and_copy_internal(token: &str) -> Result<()> {
    let token = HSTRING::from(token);

    let operation =
        SharedStorageAccessManager::RedeemTokenForFileAsync(&token)?;

    let file = operation
        .join()
        .context("Failed to redeem screenshot token")?;

    let path = file.Path()?.to_string();

    println!("Screenshot retrieved from Snipping Tool.");

    let image_bytes = std::fs::read(&path)
        .context("Failed to read captured image")?;

    // Copy image first.
    let clipboard_result =
        crate::clipboard::copy_image_to_clipboard(&image_bytes);

    // Delete the temporary file after processing.
    let delete_result = file
        .DeleteAsync(StorageDeleteOption::Default)?
        .join();

    clipboard_result?;

    delete_result.context("Could not delete temporary capture file")?;

    Ok(())
}
