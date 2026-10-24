# Bundled fonts and icons

The fonts are embedded so common English, Korean and emoji text does not wait for a system-font lookup. System fallback remains enabled for additional scripts.

- Geist regular, medium, bold and mono: [vercel/geist-font](https://github.com/vercel/geist-font/tree/75d647056bcbd9eaba3a14b1b58290d0692685d6), pinned commit `75d647056bcbd9eaba3a14b1b58290d0692685d6`. SIL Open Font License 1.1, in `Geist-OFL.txt`.
- Pretendard regular, medium and bold: [orioncactus/pretendard](https://github.com/orioncactus/pretendard/tree/7aeb0698819be2b4097dae8ec8fe6a795e5cf3ae), pinned commit `7aeb0698819be2b4097dae8ec8fe6a795e5cf3ae`. SIL Open Font License 1.1, in `Pretendard-OFL.txt`.
- Noto Sans CJK KR Regular: [notofonts/noto-cjk Sans2.004](https://github.com/notofonts/noto-cjk/tree/Sans2.004/Sans/OTF/Korean). SIL Open Font License 1.1, in `NotoSansCJK-OFL.txt`.
- Noto Emoji: [google/fonts, ofl/notoemoji](https://github.com/google/fonts/tree/main/ofl/notoemoji), downloaded 2026-10-23. SIL Open Font License 1.1, in `NotoEmoji-OFL.txt`; the exact bundled bytes are hashed below.
- Phosphor regular icon font: bytes from [egui-phosphor 0.14.0](https://crates.io/crates/egui-phosphor/0.14.0). Original [Phosphor Icons MIT license](https://github.com/phosphor-icons/web/blob/master/LICENSE), in `Phosphor-MIT.txt`. The version-specific Rust adapter is not included or linked.

| File | SHA-256 |
| --- | --- |
| `NotoSansCJKkr-Regular.otf` | `6bcb2a0703aa137e874fc2dffa85f6c21ba9a67fa329e81b8c801663af7e992a` |
| `Pretendard-Medium.otf` | `d39e50e4bb52b4993b6a4eeb821a171254745bd824446af01e1f616b89fface0` |
| `Pretendard-Bold.otf` | `2e91915fab54df71cc9598ebf608b2bdb54c6fe3c066ac61dff0bc44fca71cc7` |
| `Geist-Bold.ttf` | `3f21f02b827228c3f0c5fb230d027b5a1a263013618e8052bc1b3c5692812c88` |
| `Geist-Medium.ttf` | `3a3b36f0d0b981f4857f7f00eeef4a5ee123605575d362ab31ee7c19e3d11f2f` |
| `Geist-Regular.ttf` | `85a1c6b18a6b0a06dfe9fd4f6d6a5d4979f74ec861eaef4bc7868b5492b8a117` |
| `GeistMono-Regular.ttf` | `5a0de4b3d54ab272f76a1d8c84b7fb24c67bbec6591d5300e61c7bc10094b6c8` |
| `NotoEmoji.ttf` | `de6c18832938afc99caf132b39d6a30a19bac7f2e812e28db2535b4608d27551` |
| `Phosphor.ttf` | `06b91e022b7ee899a63efced879392a74f0bacbda54e4467e9f663220d173a10` |
| `Pretendard-Regular.ttf` | `6d0af5258997aec7354a6e340fc2325ba321c410ca48b3af858c8c3d6e92a324` |

SF Pro, Menlo and Consolas are loaded from the host's installed fonts. Their font
files are not copied into the app bundle. Geist remains available for existing themes.
