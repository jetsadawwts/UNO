# Office tab art

Pixel art for the Office tab, embedded in the binary by `crates/adapters-web/src/office.rs`.

- `floor.webp`: the office floor (1024 x 1024), generated with an AI image tool from prompts written for UNO, watermarks removed.
- `desk01..11.webp` / `chair01..11.webp`: one workstation per team member (Alpha..Lambda, in order), split into the chair (drawn behind the seated person) and the desk (drawn in front).
- `ceo-desk.webp`: the front of Uno's desk in the glass office, drawn in front of Uno.
- `people.webp`: 12 rows (Uno, Alpha..Lambda) x 7 frames (stand, walk front x2, walk back x2, sit x2), 56 x 100 px each.
- `layout.json`: where each piece goes on the floor (px), each seat, Uno's seat and the person height.

The walking routes and lounge spots are in `OFFICE_JS` (`crates/adapters-web/src/assets.rs`) and must match `floor.webp`.

Mu (`developer`, added in v0.5.22) has no workstation or sprite row yet: until the art exists Mu appears on cards and in hand-offs, not in the scene.
