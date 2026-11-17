# Interface icons

`atlas.png` packs every interface glyph into 64×64 cells (8 columns, 16 rows). Each source SVG
is scaled to 56×56 pixels, filled white and rasterized by `resvg` 0.45.1; the application tints
glyphs at draw time. No icon font, JavaScript or per-icon file is loaded at runtime.

## Discord glyphs

88 cells (87 files in `discord/`; `chats` and `thread` share `thread.svg`) are Discord's own
interface icons, so menus, toolbars and settings match the official client. They were taken on
November 18, 2026 from the icon components in Discord's public web client:
[`web.b2cb130fe43fc846.js`](https://discord.com/assets/web.b2cb130fe43fc846.js) for 75 icons,
and one lazy chunk each for `AccessibilityIcon` (`a0c18559c9916b55.js`), `ActivitiesIcon`
(`28228a662f063ed1.js`), `ArrowSmallUpIcon` (`2a969cf407131fb2.js`), `ArrowsUpDownIcon`
(`9e8a10c7271d77be.js`), `CreditCardIcon` (`f7ddd12908ea1bcd.js`), `FiltersHorizontalIcon`
(`6a2f73ae8c4b8177.js`), `KeyboardIcon` (`a8b8867267e0216b.js`), `LanguageIcon`
(`9144943ecf3dca40.js`), `PaintbrushThinIcon` (`e38fe94ec3339d91.js`), `PhoneHangUpIcon`
(`06e31c807bcf15cc.js`), `PlusLargeIcon` (`09c192e61edcaa26.js`) and `UserPlusIcon`
(`a82a9cf27175bade.js`), all under `https://discord.com/assets/`.

Each file name is the component name in kebab case without `Icon` (`MicrophoneSlashIcon` →
`microphone-slash.svg`). Path data, fill rules and the 24-unit view box are copied unchanged;
the color props are replaced with white. `circle-plus.svg` and `circle-question.svg` omit the
background circle Discord draws in `secondaryColor`, which is transparent by default.

These glyphs belong to Discord, Inc. and are not covered by Serein's MIT/Apache licenses. No
redistribution grant is documented in this repository; review the
[Discord terms](https://discord.com/terms) and obtain appropriate permission before
distributing builds containing them. Public download access is not a redistribution license.
Serein is an unofficial client and is not affiliated with or endorsed by Discord.

## Phosphor and Simple Icons

[Phosphor Icons](https://phosphoricons.com) by Helena Zhang and Tobias Fried, Copyright (c)
2023 Phosphor Icons, licensed under the [MIT License](LICENSE). Source: npm package
`@phosphor-icons/core` version 2.1.1 (repository https://github.com/phosphor-icons/core),
fetched through the jsDelivr npm mirror. The unmodified license is `LICENSE` and is staged in
both packages as `licenses/Phosphor-Icons-MIT.txt`. 28 unmodified `assets/fill/*.svg` and
`assets/bold/*.svg` files remain: fourteen third-party brand logos, `seal-check`, the eight
file-type glyphs, `folder-open`, the text context-menu glyphs (`scissors`, `clipboard-text`,
`selection-all`) and `arrows-clockwise`.

Ten brand marks Phosphor does not ship (PlayStation, Battle.net, Epic Games, League of Legends,
Riot Games, Bungie, Roblox, Crunchyroll, eBay, Bluesky) come from
[Simple Icons](https://simpleicons.org) npm package `simple-icons` version 16.30.0
(repository https://github.com/simple-icons/simple-icons), released under
[CC0 1.0](LICENSE-SIMPLE-ICONS) and fetched through the same mirror on September 11, 2026.
Their 24-unit glyphs fill the whole view box, so they are drawn at 80 % of the cell glyph size
to match Phosphor's visual weight. Brand marks remain trademarks of their owners; Simple Icons'
legal disclaimer applies. The license file is staged in both packages as
`licenses/Simple-Icons-CC0.txt`.

The `serein-mark` cell is our own artwork: the solid silhouette of the Serein chat-wave from
`assets/brand/serein-mark.svg`, trimmed to its bounding box.

## Files and regeneration

- `atlas.png`: 512×1024 RGBA, 33,996 bytes after lossless `oxipng -o max --strip all`.
  SHA-256 `ffdecc6f2ec75676381657268112d43b9e2e72d63d89362350fc8cf2e088b26d`.
- `index.tsv`: icon name, tab, zero-based cell; SHA-256
  `73c9f4253970abf6ff1f5f8206353e21182fdef70029a2552bf55eb2ea2711fa`.

Every source SVG's SHA-256 is pinned in `tools/generate-icons.py`, which refuses to build from
mismatching files; repository files are hashed with LF line endings. Cell order is fixed because
the Windows tray reads cells directly. Regenerate from the repository root:

```sh
cargo install resvg --version 0.45.1 --root /tmp/resvg-tool
python3 tools/generate-icons.py --resvg /tmp/resvg-tool/bin/resvg
```

PNG compression bytes can vary with the resvg/png versions; decoded pixels and cell indices
are deterministic. `crates/ui/src/icons.rs` resolves every `Icon` variant to its `index.tsv`
cell on first use.
