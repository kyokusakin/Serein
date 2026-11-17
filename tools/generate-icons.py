#!/usr/bin/env python3
"""Rebuild the bundled icon atlas (development only).

Reads the Discord glyphs under `assets/icons/discord/`, downloads the version-pinned Phosphor and
Simple Icons sources, verifies every file hash, composes one SVG grid and
rasterizes it with the `resvg` command-line tool. Requires `resvg` 0.45.1 on PATH or `--resvg`:

    cargo install resvg --version 0.45.1 --root /tmp/resvg-tool
    python3 tools/generate-icons.py --resvg /tmp/resvg-tool/bin/resvg
"""

import argparse
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys
import urllib.request

VERSION = "2.1.1"  # @phosphor-icons/core on npm, MIT
BASE = f"https://cdn.jsdelivr.net/npm/@phosphor-icons/core@{VERSION}"
COLUMNS, CELL, PAD = 8, 64, 4
GLYPH = CELL - 2 * PAD

# Repository-local artwork, rasterized from the repository instead of an upstream package:
# the Serein mark and the Discord interface glyphs in `assets/icons/discord/`.
BRAND_MARK = "repo:assets/brand/serein-mark.svg"

# name, source asset, SHA-256 of the unmodified SVG file. Cell order is part of the atlas
# contract: the Windows tray reads cells 4, 6 and 58 directly.
ICONS = [
    ("caret-down", "repo:assets/icons/discord/chevron-small-down.svg", "4e3bf37ec7dae16dd23adcb571a58f19fa8ecd6775f13432d52dc703117d0ab2"),
    ("caret-right", "repo:assets/icons/discord/chevron-small-right.svg", "81da3b8eed1d03a2eb5e6097f114fa81b8795259610d0cd8fcdd61656dcb8a45"),
    ("gear", "repo:assets/icons/discord/settings.svg", "2796cfba1f700347dbbf44d7fa75679a2bc95baba9815ecab7de28bc19f88633"),
    ("microphone", "repo:assets/icons/discord/microphone.svg", "789f88a4c3008052693f563a9e08306a2b20e458d3164296550c2aacd5ce0c60"),
    ("microphone-slash", "repo:assets/icons/discord/microphone-slash.svg", "995f8551e9cd15e9334e854abd7de7307f6a2ed045c8d7a7a84cd00ccea4b038"),
    ("headphones", "repo:assets/icons/discord/headphones.svg", "7da833af332a755a109fe8ace454cca570994e53f3d0d0e9b07a48357139629b"),
    ("headphones-slash", "repo:assets/icons/discord/headphones-slash.svg", "d7e8df07f9d021ea5d5298b99edb3e9044d0b290f386441d217a78b4dcbb8116"),
    ("push-pin", "repo:assets/icons/discord/pin.svg", "71530f85c570567e87bbbb880b8f23245fbbb4bd84c85742759f36d7cf9d1b2b"),
    ("users", "repo:assets/icons/discord/group.svg", "9d9eab503d0d413371a685c335ac6a374777a7187d815c40115d34062dcfb650"),
    ("user-plus", "repo:assets/icons/discord/user-plus.svg", "e2eacdbfd1ca9aea9375672b811783be15ce0e105882a2478853971e61d32f7f"),
    ("user-circle", "repo:assets/icons/discord/user-circle.svg", "a0663446c6f9a6d6e524e8c059531642a4a1c448d34787c069a0e228cac61343"),
    ("magnifying-glass", "repo:assets/icons/discord/magnifying-glass.svg", "bb19a64441cf6c4dbd9749687ab212fa2f8d9d9e55371e6c38fb745d0ea1cae0"),
    ("plus", "repo:assets/icons/discord/plus-large.svg", "1fffde3ce18d25fbf1826e431aeceda6d98756bae9ea6029da5442a3c8785d49"),
    ("plus-circle", "repo:assets/icons/discord/circle-plus.svg", "c09519bfcd7628ac0856d1cce63c3aaeb6cdd5dbc2e443c684ddea9316b5c846"),
    ("x", "repo:assets/icons/discord/x-large.svg", "bfc53b2f329e489f0e7dda960357f1679dc2327149213d7d5598080864ce381b"),
    ("smiley", "repo:assets/icons/discord/emoji-slightly-smiling-face.svg", "1ec9e90b606eef450357be8fbbbfad65f770cfd08f27ecc04097597e2057c174"),
    ("bell", "repo:assets/icons/discord/bell.svg", "e876e85a88edd1d97c0964f460f757aaaa39a6c49b35df3c4454a2b9cd80a51f"),
    ("phone", "repo:assets/icons/discord/phone-call.svg", "7517a982e128014d72c08270e1d9e1a5260b3f7daa455d30af18f11e5f5ee73d"),
    ("phone-call", "repo:assets/icons/discord/phone.svg", "33605438a11d8d6b9dd7390ca1a4a7c8819e3d79931f800d7793f0d27afaf88a"),
    ("phone-disconnect", "repo:assets/icons/discord/phone-hang-up.svg", "7587f1296a7446cbb9d112bb2c55732e2399d545ecc10bda13f689d291e415c6"),
    ("video-camera", "repo:assets/icons/discord/video.svg", "b69470f3446d814c8b27af580c28d5cf4c2eacae2a88913cbca051bd4350218d"),
    ("video-camera-slash", "repo:assets/icons/discord/video-slash.svg", "a6d3481738281c4546259970420f55bde02b2d849f0ccd553a2cac5e441ab3fd"),
    ("monitor-arrow-up", "repo:assets/icons/discord/screen-arrow.svg", "f5ab78188268d427d3b6bc9fbe6604ace7b212d28a76b9895e3492546a277f56"),
    ("rocket-launch", "repo:assets/icons/discord/activities.svg", "4680351a09fc5ff7d183d3ce0a7cbfcfad7ffc4e59bb000811bc9bafb984c905"),
    ("waveform", "repo:assets/icons/discord/soundboard.svg", "36ed2a18910fad83fa9ec7d02a3c2b472d68826bf84561e7327e165cdae01f37"),
    ("arrow-bend-up-left", "repo:assets/icons/discord/arrow-angle-left-up.svg", "6d2b351fbb63582c8cf829160ec1208b5fb80b471d803a32908f044c4592b449"),
    ("pencil-simple", "repo:assets/icons/discord/pencil.svg", "d6078761ceebaf9e05779c885fa6705daded0833d2792a5cc4acc8fdb8e8c866"),
    ("dots-three", "repo:assets/icons/discord/more-horizontal.svg", "6a25e38bda8ece5961b826736caa2ecafc205f1f0f7d2e6fae73446f114e3c44"),
    ("tray", "repo:assets/icons/discord/inbox.svg", "03f61c2e726bd403dcc5af90d49f568d7a596fbca1632830e26bbb123244d704"),
    ("question", "repo:assets/icons/discord/circle-question.svg", "1b59c1f0adccc4424f71255490f8ea352909097634b323a28d38ce0d726d2682"),
    ("arrow-clockwise", "repo:assets/icons/discord/refresh.svg", "c0a0a6344dba78f39790f21703cb5650909f8d2db87278295ff2be72db982610"),
    ("chats", "repo:assets/icons/discord/thread.svg", "99e4b49d90b586a7dc4316e2d349a6926a9dd6330e7837e3490afdfad1ffd122"),
    ("speaker-high", "repo:assets/icons/discord/voice-normal.svg", "98bf1bb0846f3f9c3040ac335ec7b3d90e0569d74887bcb491ebf76f2c6b1f9f"),
    ("hash", "repo:assets/icons/discord/text.svg", "aadeeffa12fb94b1cb403e2df1b119affd7c08e1826497d574c1e67b5895af70"),
    ("chat-centered-text", "repo:assets/icons/discord/forum.svg", "afbf565c8f2ddcd598afb0a72db79b3d33e8d50d3ae34cb8483940a8f5528ef4"),
    ("paper-plane-right", "repo:assets/icons/discord/send-message.svg", "413382409490a0a9a4c1cdd43461bb09af5d77b9ea4280443c1319d834c83525"),
    ("arrow-square-out", "repo:assets/icons/discord/link-external-small.svg", "fd6f2226f49b3413734ba4285b18c2af67fdb0b2e5940179aae00cc188cd6928"),
    ("github-logo", "fill/github-logo-fill.svg", "35b2036490005bcb95a68d07d43d441ae585c6c6dde0e902bb3daf0ddbc615b9"),
    ("twitch-logo", "fill/twitch-logo-fill.svg", "df5adaa427994af42c961715d6e75f496b1832606a68cbd8090d09c25a4e8361"),
    ("steam-logo", "fill/steam-logo-fill.svg", "1715fdfe47e32c6cbff76fd80dcb47c1e5a704b2067412dd64dff20b8d802a11"),
    ("spotify-logo", "fill/spotify-logo-fill.svg", "50360bee2998f48367780b6f1637d208169acb0d3384e27e145f29b456b73bd4"),
    ("youtube-logo", "fill/youtube-logo-fill.svg", "fce097e65d0df0cb2bb84cd282bb6cc08ccb47ec18d41b7c22870830257065be"),
    ("x-logo", "fill/x-logo-fill.svg", "b760dcebe932bafc69784799231eb8ce988dc33fa5d68d20e706508b24264526"),
    ("reddit-logo", "fill/reddit-logo-fill.svg", "f6e1c29f724ad06cf8ab8c41e26a72a482e6187724ac7cc91fe880b9c499873d"),
    ("facebook-logo", "fill/facebook-logo-fill.svg", "4ef00877d0348c4db3eb93d01d36d9b82528ca2d635d0e3a5e51515299cf99f7"),
    ("instagram-logo", "fill/instagram-logo-fill.svg", "24444c671b0c50580b39138453762225d4d53b7d04c978244b8027a84c719746"),
    ("tiktok-logo", "fill/tiktok-logo-fill.svg", "aafb59c3c1902035736bc955e1b4485e46fd8b633bb2cd2e046937cc1b26c5cb"),
    ("paypal-logo", "fill/paypal-logo-fill.svg", "b49cb17698ca5c2d807577413f992b9d0050b2a94ca4c812c6e6441a32f9f820"),
    ("amazon-logo", "fill/amazon-logo-fill.svg", "96d475f1eaed1733e9c6dcae74b52900be7e399b128917a8f50d89d773041491"),
    ("mastodon-logo", "fill/mastodon-logo-fill.svg", "f9d1bf489b16a34763aa7f8e8de62f72812cb9888fd92bbc47eeac33d1426122"),
    ("skype-logo", "fill/skype-logo-fill.svg", "bd264055d7317fb0cfe3a39ae17028d2a5447dc3a204a999c5e5fbebed423e28"),
    ("game-controller", "repo:assets/icons/discord/game-controller.svg", "44f8e28ff41e575cd6d605f7c8dc16a5bb025c3ef91133fc4ba2c667cb80f62d"),
    ("television", "repo:assets/icons/discord/tv.svg", "515b0ca68410aa8cf81cb18d1dd1beeb3aaa754cea943f04e0115576e38ee27b"),
    ("globe", "repo:assets/icons/discord/globe-earth.svg", "d6661669d3fba175122f172138384531cbbca38f22796d13c471b97dae3d6a40"),
    ("link", "repo:assets/icons/discord/link.svg", "768709fafc601aed83905b4e08fea798af268584cf12ed4a430fbc833e664a34"),
    ("copy", "repo:assets/icons/discord/copy.svg", "7dc0f893f3fa22a0e9302afd3640e070448a456370391d551ef81481b2202dec"),
    ("seal-check", "fill/seal-check-fill.svg", "27702a62622ac4156d2a18b10fc7c526b3b2b9f782741e2cb07aea6964d71132"),
    ("calendar-blank", "repo:assets/icons/discord/calendar.svg", "efd45a80f2e06a2a8b94cb15158e6aa55888bdfc9c775601ad5f9a2d0916551e"),
    ("serein-mark", BRAND_MARK, "131359276c0f4abe7f97f99f0ad73cad88fec4d9845ad01bcdece940a089dc3e"),
    ("file", "fill/file-fill.svg", "d6fe00691e45b5e9b87ccc5a8fc9022485935408167ab1234817915df4fd1ca4"),
    ("file-image", "fill/file-image-fill.svg", "0266eb983ed5cec9152d76691152123b92b2cb84d8c139c37793d0870df3f601"),
    ("file-pdf", "fill/file-pdf-fill.svg", "12622b293b9a1efa1f56969ca74ab88085d388c4be926f81cfcb49406a8da05d"),
    ("file-zip", "fill/file-zip-fill.svg", "3b4355954bef19b2572639dcb619e0e81513118dd76c3c2fc8bab6d9a23c9032"),
    ("file-text", "fill/file-text-fill.svg", "8c6f36452a441d39886ad7929778c2a90ea66cca519490da4918e7da7bade242"),
    ("file-code", "fill/file-code-fill.svg", "127044e7395681d5306c7553cc2287e16c63f9d4578f4f6fe7710f277e5b7624"),
    ("file-audio", "fill/file-audio-fill.svg", "835c6bfde42e68d716db3dd4db0e02843f6a7bd704a4377287a4b022d80ace7a"),
    ("file-video", "fill/file-video-fill.svg", "2940d83bf3560f389d6a826ca639741a9098d5bfe8910ea371fdc4c83790b827"),
    ("trash", "repo:assets/icons/discord/trash.svg", "412f1c8b629e2949c4699bfa4f10d631b5297106a89c9047b66c972a584e250b"),
    ("arrow-down", "repo:assets/icons/discord/arrow-small-down.svg", "6a366eb401e0749e4dcc1fc99d09b4130537527b54bf3e2dcd234cf3e8864bbb"),
    ("arrow-up", "repo:assets/icons/discord/arrow-small-up.svg", "cbccb4b65957a86cab209cc5c2ad3608a41e84ff7f0fc7f402b0308456aaa946"),
    ("check", "repo:assets/icons/discord/checkmark-large.svg", "a61b14d48c9f8c62ccec788ccc8fbd9668793737de4c97c941bbc133ba9a9cce"),
    ("gif", "repo:assets/icons/discord/gif.svg", "b35663f3e765f67ebe2540aa6d0dc9d86b1c31cad1d4922191ae4ea5c2b6a887"),
    ("star", "repo:assets/icons/discord/star-outline.svg", "9206c98210c6d32e5a7ca6ae4a1391ceb669663bdfa14550d70977f88de0318c"),
    ("star-fill", "repo:assets/icons/discord/star.svg", "2c6db9014d6b7ad2286aff9957364fa1ef58742556fc5fb694f1d5044c34fe73"),
    ("fire", "repo:assets/icons/discord/fire.svg", "eda3dfe974bce2e978294e76660ed2c6f3332dcd3458f6b6bd957ff1b3f418be"),
    ("arrow-left", "repo:assets/icons/discord/arrow-small-left.svg", "59f86dca9514cd3b0fb949f38a11e6799dbb729c1ad0051d09a9bdb2baa98664"),
    ("folder", "repo:assets/icons/discord/folder.svg", "d87ba163b758cf9c1239f3ea0741606990cd6636ee760778d08f3a2882eb63b0"),
    ("folder-open", "fill/folder-open-fill.svg", "f9e57c3d40ab536915f8302a4390405f5660ee7b7275a3f80b384d977c59373e"),
    ("caret-left", "repo:assets/icons/discord/chevron-small-left.svg", "6255ba280807e2b5b3168b0a799d9d87e504c4229d91beffb3d8d8cb3e511c11"),
    ("download-simple", "repo:assets/icons/discord/download.svg", "215356068135026757dcb24c58d38cb9ed5ec9a8ae83945a9ca57cb00e211169"),
    ("arrow-right", "repo:assets/icons/discord/arrow-small-right.svg", "60a39702c735e896884969468b7b70a1fd1baec2f0005a139cb61620e3f24450"),
    ("image", "repo:assets/icons/discord/image.svg", "c79ff6b1b583b5705f4f96d07f21057891175e230b7d0653b1819f69c211b849"),
    ("sparkle", "repo:assets/icons/discord/sparkles.svg", "abb4eb14387ac40768ba5a47f41e3d9acec872a0adab808f98be9eb6dfb443df"),
    ("compass", "repo:assets/icons/discord/compass.svg", "b5d312ab62e9d34adaef0ebeb8038250d4db7a7441c432c01629930376749522"),
    ("megaphone-simple", "repo:assets/icons/discord/announcements.svg", "3f08ef521b9f27b35ce7289f44db476ced95adce1ec1ab4bafa193ccccd50521"),
    ("shield-warning", "repo:assets/icons/discord/warning.svg", "c3390bd0cb70a81ffa73a20e5f4dac9d8e44a061b3be8a7d2d6babac6272ce06"),
    ("crown", "repo:assets/icons/discord/crown.svg", "2e634a195aead4e1d4418f44401cf2a6dd72e4c4899ccd1923a9b524c2b45612"),
    ("chart-bar", "repo:assets/icons/discord/analytics.svg", "edf28c5cba3657b667e487d0a3df7a45819575cd456a4880328ba8337833c920"),
    ("shopping-cart-simple", "repo:assets/icons/discord/shop.svg", "b6082ae06dbe88474e9c77e87b89fb10908286ce5389bedce6fa1298df00c76e"),
    ("lock-simple", "repo:assets/icons/discord/lock.svg", "e4faca460aaf61876915dbacf9d29994abb836cb4aab9fc43bfa2cecfda3e983"),
    ("eye-slash", "repo:assets/icons/discord/eye-slash.svg", "254495e650909bfe88c06ab64d823f41138de7bcd546c2bbeb1a0463448e3d07"),
    ("sliders-horizontal", "repo:assets/icons/discord/filters-horizontal.svg", "0df2acc035a76dfc8f2089c1779d4d5e8683392d19f34e3b53c52c16bafa7ff5"),
    ("arrows-down-up", "repo:assets/icons/discord/arrows-up-down.svg", "ef78403eb431d269acc3afe9bd56e70b5fceab8437a609d1c58c45b0d213e80c"),
    ("thread", "repo:assets/icons/discord/thread.svg", "99e4b49d90b586a7dc4316e2d349a6926a9dd6330e7837e3490afdfad1ffd122"),
    ("device-mobile", "repo:assets/icons/discord/mobile-phone.svg", "57e16a0bbe7088851c053548427a521bd356f4aaee2eb771ab1589a4019eeed3"),
    ("hard-drives", "repo:assets/icons/discord/server.svg", "8221996e8ea490c8f2dc081eb2870c5ca4e4d62a249f0d30ae7e30199adff658"),
    ("corners-out", "repo:assets/icons/discord/fullscreen-enter.svg", "3fbbfd841584f89dc06c8a891663335844b2f453e146b501f5daef0b07d959b7"),
    ("scissors", "bold/scissors-bold.svg", "10d7c433d8284923caa353edbfa984be5c9b12aab331c36d3f250f3bd7656a1d"),
    ("clipboard-text", "bold/clipboard-text-bold.svg", "3d1f047a9ea53283cada888642cc9427e3ec7ac766beeefee19581454b4b26b4"),
    ("selection-all", "bold/selection-all-bold.svg", "a5db79d398382ba9c9bf0abe75290e46cf11c41464be439b66c8e3ee30a716a3"),
    ("monitor-x", "repo:assets/icons/discord/screen-x.svg", "8de6dea5591970fa80529309577836c7b74a6a1efa0a7beeaa31dc7484e1e36b"),
    ("user", "repo:assets/icons/discord/user.svg", "e0108342c023a805e1ea31a01b7d9a9c55453dbd46b066de1583b7cdb269d5e2"),
    ("chat-circle-dots", "repo:assets/icons/discord/chat.svg", "f3bdc69a5a1d13cc677df29c4e5488b11b44c1a05ab6b61740a89163414a8c89"),
    ("tire", "repo:assets/icons/discord/nitro-wheel.svg", "f4467ebcb1d84f34a215a257e0092a7194653265fbcb9fd55b264a6853a08955"),
    ("diamond", "repo:assets/icons/discord/boost-gem.svg", "512ed583decdc8ef48f1ae0dfe2299c218142933c2757d39e6503a8f85b5ee46"),
    ("arrows-clockwise", "bold/arrows-clockwise-bold.svg", "f79592c23ce2e13807dcf27ed5de18aa1484beb9a976b0c0bfd81ee280198b55"),
    ("gift", "repo:assets/icons/discord/gift.svg", "923678856b4a4a5399890dbd9ff37ca8bdfaab3c181742747aefd59a11560aaf"),
    ("credit-card", "repo:assets/icons/discord/credit-card.svg", "789d41276d4f8dd44a1ef48d2b638862491eb310ce62c6fdb65791193ecec3c0"),
    ("palette", "repo:assets/icons/discord/paint-palette.svg", "5b9802b8b8753632c3025cb74d24a5bd6787b7420fd1cc6b0d4be86896f668b1"),
    ("person-arms-spread", "repo:assets/icons/discord/accessibility.svg", "efbdb4c6826aec668fe5a1f8b3aa6c185e861c4499075fbff9ee86600ec9b88b"),
    ("keyboard", "repo:assets/icons/discord/keyboard.svg", "51433e3870c1485afc70447d9285995a753c8d6df67165570e3cb8f099dacdf8"),
    ("translate", "repo:assets/icons/discord/language.svg", "9e5a178f838142b2539d31e0677f3c39c6974496c96975eda5a885cd521b8400"),
    ("user-focus", "repo:assets/icons/discord/shield-user.svg", "645529db5f87d2a160a84569579df4f69dc4ca1ed300cdf0866f89f9458c6791"),
    ("code", "repo:assets/icons/discord/angle-brackets.svg", "d6e88e92db1e10a3906c4e8a0b54b46b6b0be2bc1cd9fb3c8fe702aba028459a"),
    ("sign-out", "repo:assets/icons/discord/door-exit.svg", "fae777acfe07d0523d2cfa02de83e67f2d9650848b4d27cfeaa81a7f55473991"),
    ("puzzle-piece", "repo:assets/icons/discord/apps.svg", "432f2a06cbd46c7033e6e74a74669087241c6fff90fb9753bdab1d688a845954"),
    ("paint-brush", "repo:assets/icons/discord/paintbrush-thin.svg", "a394d45779c706f7f75bc20403d4beb823d981addc8e764bce32c9cd0a592ef8"),
]
LICENSE_SHA256 = "ddbe6082ec3cf979db47e5af549d2849c5d6182b3e005ef91ce1dbb9eb122f11"

# Brand marks Phosphor does not ship come from Simple Icons (CC0 1.0). Their 24-unit glyphs
# fill the whole view box, so they are drawn at Phosphor's visual size inside the cell.
SIMPLE_VERSION = "16.30.0"  # simple-icons on npm, CC0-1.0
SIMPLE_BASE = f"https://cdn.jsdelivr.net/npm/simple-icons@{SIMPLE_VERSION}"
SIMPLE_ICONS = [
    ("playstation", "icons/playstation.svg", "b68b4d7b63443759b9c4d77a5501c5758eeae06a6d594d278d5eb6cb4d3dbbe4"),
    ("battle-net", "icons/battledotnet.svg", "78206c9c5e7fd24803cd50cd5c71c3bb1b02def42ca4f5d186e318eb9e247b7e"),
    ("epic-games", "icons/epicgames.svg", "a19b1eb5a46edc11a7dc7f1ce6fa1701ea4e4cf451feec88441c523f4b50cde3"),
    ("league-of-legends", "icons/leagueoflegends.svg", "b653d9c5733c71613fa0367c09c9d419df2f0e7ef91cbeab6b93a2e20d5f223b"),
    ("riot-games", "icons/riotgames.svg", "b80c5880b88b8e489b2da7753bed612b3ce6b2948a3f995ad5d31050f443b520"),
    ("bungie", "icons/bungie.svg", "92c3b473805eaffdc7092cb8e8c1500ccac1bea19c129d46b026c22af6000db7"),
    ("roblox", "icons/roblox.svg", "9245b2f23fde91a5ef34f36aea32c1bf08a1a1254c08b86e7f060a4f72986234"),
    ("crunchyroll", "icons/crunchyroll.svg", "3b9c3d87339e18ec09f25e0c3eaffdc5ad4630df3d108fcd907e1b64c4cd13ea"),
    ("ebay", "icons/ebay.svg", "846e8d8ac6cea49766e7c62e095e739d115ed355f9b63bf46067f14c9343c745"),
    ("bluesky", "icons/bluesky.svg", "49752973164fbbf4464fbb4776f011c1eff207e5d7ad9254e031af025814eb75"),
]
SIMPLE_LICENSE_SHA256 = "9046848b63a5c92bff14e4accca80bd987e0623b74adf9226ce5198d312b79d5"
SIMPLE_SCALE = 0.8  # Phosphor fill glyphs span roughly 205 of their 256 units.


def fetch(path, base=BASE):
    with urllib.request.urlopen(f"{base}/{path}", timeout=60) as response:
        return response.read(1024 * 1024)


def inner_svg(svg):
    start = svg.index(">", svg.index("<svg")) + 1
    end = svg.rindex("</svg>")
    return svg[start:end]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--resvg", default="resvg", help="path to the resvg executable")
    parser.add_argument("--print-hashes", action="store_true", help="print SHA-256 values instead of checking")
    args = parser.parse_args()
    destination = Path(__file__).resolve().parents[1] / "assets" / "icons"
    destination.mkdir(parents=True, exist_ok=True)

    license_text = fetch("LICENSE")
    if args.print_hashes:
        print("LICENSE", hashlib.sha256(license_text).hexdigest())
    elif hashlib.sha256(license_text).hexdigest() != LICENSE_SHA256:
        raise ValueError("LICENSE SHA-256 mismatch")

    simple_license = fetch("LICENSE.md", SIMPLE_BASE)
    if args.print_hashes:
        print("SIMPLE LICENSE", hashlib.sha256(simple_license).hexdigest())
    elif hashlib.sha256(simple_license).hexdigest() != SIMPLE_LICENSE_SHA256:
        raise ValueError("Simple Icons LICENSE SHA-256 mismatch")

    sources = [(name, asset, sha256, False) for name, asset, sha256 in ICONS]
    sources += [(name, asset, sha256, True) for name, asset, sha256 in SIMPLE_ICONS]
    names = [name for name, _, _, _ in sources]
    assert len(set(names)) == len(names)
    rows = (len(sources) + COLUMNS - 1) // COLUMNS
    width, height = COLUMNS * CELL, rows * CELL
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">']
    index = []
    root = Path(__file__).resolve().parents[1]
    for cell, (name, asset, sha256, simple) in enumerate(sources):
        local = asset.startswith("repo:")
        if local:
            # Hashes pin the LF blob Git stores, whatever the checkout's line endings.
            svg = (root / asset.removeprefix("repo:")).read_bytes().replace(b"\r\n", b"\n")
        else:
            svg = fetch(asset, SIMPLE_BASE) if simple else fetch(f"assets/{asset}")
        digest = hashlib.sha256(svg).hexdigest()
        if args.print_hashes:
            print(name, asset, digest)
        elif digest != sha256:
            raise ValueError(f"{asset} SHA-256 mismatch: {digest}")
        text = svg.decode("utf-8")
        body = inner_svg(text)
        x = (cell % COLUMNS) * CELL + PAD
        y = (cell // COLUMNS) * CELL + PAD
        if local:
            # Local glyphs fill their own view box; honour its offset and size.
            box = text.split('viewBox="', 1)[1].split('"', 1)[0].split()
            left, top, size = float(box[0]), float(box[1]), float(box[2])
            scale = GLYPH / size
            x, y = x - left * scale, y - top * scale
        elif simple:
            assert 'viewBox="0 0 24 24"' in text, asset
            inset = GLYPH * (1 - SIMPLE_SCALE) / 2
            x, y, scale = x + inset, y + inset, GLYPH * SIMPLE_SCALE / 24
        else:
            assert 'viewBox="0 0 256 256"' in text, asset
            scale = GLYPH / 256
        parts.append(f'<g transform="translate({x:.6f} {y:.6f}) scale({scale:.6f})" fill="#fff">{body}</g>')
        index.append((name, cell))
    parts.append("</svg>")
    if args.print_hashes:
        return
    atlas_svg = destination / "atlas.svg"
    atlas_svg.write_text("".join(parts), encoding="utf-8")
    subprocess.run([args.resvg, str(atlas_svg), str(destination / "atlas.png")], check=True)
    atlas_svg.unlink()
    # Lossless; resvg writes an unoptimized PNG about 4x larger. Install with `cargo install oxipng`.
    if shutil.which("oxipng"):
        subprocess.run(["oxipng", "-o", "max", "--strip", "all", "-q", destination / "atlas.png"], check=True)
    else:
        print("oxipng not found; atlas.png left at resvg compression", file=sys.stderr)
    (destination / "index.tsv").write_text("".join(f"{name}\t{cell}\n" for name, cell in index), encoding="utf-8")
    (destination / "LICENSE").write_bytes(license_text)
    (destination / "LICENSE-SIMPLE-ICONS").write_bytes(simple_license)
    print(f"{len(sources)} icons; {width}x{height}; atlas {(destination / 'atlas.png').stat().st_size} bytes")
    for file in ["atlas.png", "index.tsv", "LICENSE", "LICENSE-SIMPLE-ICONS"]:
        print(file, hashlib.sha256((destination / file).read_bytes()).hexdigest())


if __name__ == "__main__":
    main()
