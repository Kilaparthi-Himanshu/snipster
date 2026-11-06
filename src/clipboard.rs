
use anyhow::{anyhow, Context, Result};

use windows::Win32::{
    Foundation::{GlobalFree, HANDLE},
    System::{
        DataExchange::{
            CloseClipboard,
            EmptyClipboard,
            OpenClipboard,
            SetClipboardData,
        },
        Memory::{
            GlobalAlloc,
            GlobalLock,
            GlobalUnlock,
            GMEM_MOVEABLE,
        },
    },
};

const CF_DIB: u32 = 8;

pub fn copy_image_to_clipboard(image_bytes: &[u8]) -> Result<()> {
    let image = image::load_from_memory(image_bytes)
        .context("Could not decode captured screenshot")?
        .to_rgb8();

    let width = image.width();
    let height = image.height();

    if width == 0 || height == 0 {
        return Err(anyhow!("Captured image has invalid dimensions"));
    }

    let width_i32 = i32::try_from(width)?;
    let height_i32 = i32::try_from(height)?;

    let row_bytes = (width as usize)
        .checked_mul(3)
        .ok_or_else(|| anyhow!("Image dimensions overflow"))?;

    let stride = row_bytes
        .checked_add(3)
        .ok_or_else(|| anyhow!("Image dimensions overflow"))?
        & !3;

    let pixel_bytes = stride
        .checked_mul(height as usize)
        .ok_or_else(|| anyhow!("Image dimensions overflow"))?;

    let total_size = 40usize
        .checked_add(pixel_bytes)
        .ok_or_else(|| anyhow!("Image buffer size overflow"))?;

    let pixel_bytes_u32 = u32::try_from(pixel_bytes)?;

    let mut dib = vec![0u8; total_size];

    // BITMAPINFOHEADER
    dib[0..4].copy_from_slice(&40u32.to_le_bytes());
    dib[4..8].copy_from_slice(&width_i32.to_le_bytes());
    dib[8..12].copy_from_slice(&(-height_i32).to_le_bytes());
    dib[12..14].copy_from_slice(&1u16.to_le_bytes());
    dib[14..16].copy_from_slice(&24u16.to_le_bytes());
    dib[16..20].copy_from_slice(&0u32.to_le_bytes());
    dib[20..24].copy_from_slice(&pixel_bytes_u32.to_le_bytes());

    // Convert RGB pixels into BGR pixels.
    for y in 0..height as usize {
        let source_row = y * width as usize * 3;
        let destination_row = 40 + y * stride;

        for x in 0..width as usize {
            let src = source_row + x * 3;
            let dst = destination_row + x * 3;

            dib[dst] = image.as_raw()[src + 2];
            dib[dst + 1] = image.as_raw()[src + 1];
            dib[dst + 2] = image.as_raw()[src];
        }
    }

    unsafe {
        OpenClipboard(None)
            .context("Could not open Windows clipboard")?;

        let result = set_dib_data(&dib);
        let close_result = CloseClipboard();

        result?;
        close_result.context("Could not close Windows clipboard")?;
    }

    Ok(())
}

unsafe fn set_dib_data(dib: &[u8]) -> Result<()> {
    let memory = unsafe {
        GlobalAlloc(GMEM_MOVEABLE, dib.len())
            .context("Could not allocate clipboard memory")?
    };

    let pointer = unsafe { GlobalLock(memory) };

    if pointer.is_null() {
        unsafe {
            let _ = GlobalFree(Some(memory));
        }

        return Err(anyhow!("Could not lock clipboard memory"));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            dib.as_ptr(),
            pointer as *mut u8,
            dib.len(),
        );

        let _ = GlobalUnlock(memory);

        if let Err(error) = EmptyClipboard() {
            let _ = GlobalFree(Some(memory));
            return Err(error.into());
        }

        if let Err(error) = SetClipboardData(
            CF_DIB,
            Some(HANDLE(memory.0)),
        ) {
            let _ = GlobalFree(Some(memory));
            return Err(error.into());
        }
    }

    // Windows owns the allocation after successful SetClipboardData.
    Ok(())
}
