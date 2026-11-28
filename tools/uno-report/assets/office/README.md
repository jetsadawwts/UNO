# Office tab art

Pixel art for the Office tab, embedded in the binary by `crates/adapters-web/src/office.rs`.

- `floor.webp`: the office floor (1024 x 1024), generated with an AI image tool from prompts written for UNO, watermarks removed.
- `desk01..12.webp` / `chair01..12.webp`: one workstation per team member (Alpha..Mu, in order), split into the chair (drawn behind the seated person) and the desk (drawn in front).
- `ceo-desk.webp`: the front of Uno's desk in the glass office, drawn in front of Uno.
- `people.webp`: 13 rows (Uno, Alpha..Mu) x 7 frames (stand, walk front x2, walk back x2, sit x2), 56 x 100 px each.
- `layout.json`: where each piece goes on the floor (px), each seat, Uno's seat and the person height.

The walking routes and lounge spots are in `OFFICE_JS` (`crates/adapters-web/src/assets.rs`) and must match `floor.webp`.

Mu's desk (`desk12`, v0.5.23) reuses Kappa's desk body with Mu's own items placed on it, so it matches the others in size and colour.
