//! Clipboard diagnostics preserve every original item and data representation in memory.
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSPasteboard;
use objc2_app_kit::NSPasteboardItem;
use objc2_app_kit::NSPasteboardWriting;
use objc2_foundation::NSArray;

pub(super) struct ClipboardBackup {
    items: Vec<Retained<NSPasteboardItem>>,
}

impl ClipboardBackup {
    pub fn capture() -> Result<Self, String> {
        let board = NSPasteboard::generalPasteboard();
        let mut items = Vec::new();
        if let Some(original) = board.pasteboardItems() {
            for item in original.iter() {
                let backup = NSPasteboardItem::new();
                for kind in item.types().iter() {
                    let data = item
                        .dataForType(&kind)
                        .ok_or("clipboard representation unavailable")?;
                    if !backup.setData_forType(&data, &kind) {
                        return Err("could not preserve the clipboard".into());
                    }
                }
                items.push(backup);
            }
        }
        Ok(Self { items })
    }
}

impl Drop for ClipboardBackup {
    fn drop(&mut self) {
        let board = NSPasteboard::generalPasteboard();
        let items: Vec<&ProtocolObject<dyn NSPasteboardWriting>> = self
            .items
            .iter()
            .map(|item| ProtocolObject::from_ref(&**item))
            .collect();
        board.clearContents();
        if !items.is_empty() && !board.writeObjects(&NSArray::from_slice(&items)) {
            eprintln!("Could not restore the diagnostic clipboard backup");
        }
    }
}
