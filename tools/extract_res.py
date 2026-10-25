#!/usr/bin/env python3
"""Extract resources from a 16-bit Windows .RES file (Windows 3.x format).

    python3 extract_res.py ROCK20.RES out_dir

Writes:
  bitmaps/<name>.png      every BITMAP (stored as a DIB without file header)
  icons/<name>.ico/.png   every icon group, rebuilt as a .ico and a PNG preview
  cursors/<name>.cur      every cursor group
  menus/<name>.json       MENU templates
  dialogs/<name>.json     DIALOG templates
  strings.json            STRINGTABLE entries
  manifest.json           a list of everything found

Needs Pillow (pip install pillow).
"""
import io
import json
import os
import struct
import sys

from PIL import Image

TYPES = {1: "CURSOR", 2: "BITMAP", 3: "ICON", 4: "MENU", 5: "DIALOG",
         6: "STRING", 7: "FONTDIR", 8: "FONT", 9: "ACCELERATOR",
         10: "RCDATA", 12: "GROUP_CURSOR", 14: "GROUP_ICON"}


def read_id(d, p):
    """A type or name: 0xFF + ordinal word, or a NUL-terminated string."""
    if d[p] == 0xFF:
        return struct.unpack_from("<H", d, p + 1)[0], p + 3
    e = d.index(b"\0", p)
    return d[p:e].decode("latin1"), e + 1


def cstr(d, p):
    e = d.index(b"\0", p)
    return d[p:e].decode("latin1"), e + 1


def parse_res(d):
    p = 0
    while p < len(d):
        typ, p = read_id(d, p)
        name, p = read_id(d, p)
        flags, size = struct.unpack_from("<HI", d, p)
        p += 6
        yield typ, name, flags, d[p:p + size]
        p += size


def dib_to_image(dib):
    hs, w, h, planes, bpp, comp, isz, _, _, clr_used, _ = struct.unpack_from("<IiiHHIIiiII", dib)
    ncol = clr_used or (1 << bpp if bpp <= 8 else 0)
    off = 14 + hs + ncol * 4
    bmp = struct.pack("<2sIHHI", b"BM", 14 + len(dib), 0, 0, off) + dib
    return Image.open(io.BytesIO(bmp)), (w, h, bpp)


def parse_menu(data):
    # MENUHEADER (version, header size) then items; MF_POPUP=0x10, MF_END=0x80
    p = 4

    def items(p):
        out = []
        while True:
            fl = struct.unpack_from("<H", data, p)[0]
            p += 2
            if fl & 0x10:
                text, p = cstr(data, p)
                sub, p = items(p)
                out.append({"popup": text, "flags": fl, "items": sub})
            else:
                iid = struct.unpack_from("<H", data, p)[0]
                text, p = cstr(data, p + 2)
                out.append({"id": iid, "text": text, "flags": fl})
            if fl & 0x80:
                return out, p
    return items(p)[0]


def parse_dialog(data):
    style, count, x, y, cx, cy = struct.unpack_from("<IBhhhh", data)
    p = 13

    def sz_or_ord(p):
        if data[p] == 0xFF:
            return data[p + 1], p + 2
        return cstr(data, p)
    menu, p = sz_or_ord(p)
    cls, p = sz_or_ord(p)
    caption, p = cstr(data, p)
    dlg = {"style": hex(style), "rect": [x, y, cx, cy], "menu": menu,
           "class": cls, "caption": caption, "controls": []}
    if style & 0x40:  # DS_SETFONT
        pt = struct.unpack_from("<H", data, p)[0]
        face, p = cstr(data, p + 2)
        dlg["font"] = [pt, face]
    for _ in range(count):
        cx_, cy_, cw, ch, cid, cstyle = struct.unpack_from("<hhhhHI", data, p)
        p += 14
        if data[p] & 0x80:
            ccls = {0x80: "BUTTON", 0x81: "EDIT", 0x82: "STATIC", 0x83: "LISTBOX",
                    0x84: "SCROLLBAR", 0x85: "COMBOBOX"}.get(data[p], data[p])
            p += 1
        else:
            ccls, p = cstr(data, p)
        text, p = sz_or_ord(p)
        extra = data[p]
        p += 1 + extra
        dlg["controls"].append({"class": ccls, "id": cid, "text": text,
                                "rect": [cx_, cy_, cw, ch], "style": hex(cstyle)})
    return dlg


def main(src, out):
    d = open(src, "rb").read()
    os.makedirs(os.path.join(out, "bitmaps"), exist_ok=True)
    manifest, icons, cursors, strings = [], {}, {}, {}
    entries = list(parse_res(d))
    for typ, name, flags, data in entries:
        tn = TYPES.get(typ, str(typ))
        info = {"type": tn, "name": name, "flags": hex(flags), "size": len(data)}
        if typ == 2:
            im, (w, h, bpp) = dib_to_image(data)
            im.convert("RGB").save(os.path.join(out, "bitmaps", f"{str(name).lower()}.png"))
            info.update(width=w, height=h, bpp=bpp)
        elif typ == 3:
            icons[name] = data
        elif typ == 1:
            cursors[name] = data
        elif typ == 6:
            # 16 length-prefixed strings per block; block n holds ids (n-1)*16..
            p = 0
            for i in range(16):
                n = data[p]
                if n:
                    strings[(name - 1) * 16 + i] = data[p + 1:p + 1 + n].decode("latin1")
                p += 1 + n
        manifest.append(info)

    for typ, name, flags, data in entries:
        if typ in (14, 12):
            _, rtype, count = struct.unpack_from("<HHH", data)
            members, blobs = [], []
            for i in range(count):
                if typ == 14:
                    w, h, cc, _, planes, bc, size, iid = struct.unpack_from("<BBBBHHIH", data, 6 + i * 14)
                    blob = icons[iid]
                    members.append((w, h, cc, planes, bc, blob))
                else:
                    w, h, planes, bc, size, iid = struct.unpack_from("<HHHHIH", data, 6 + i * 14)
                    blob = cursors[iid]
                    hx, hy = struct.unpack_from("<HH", blob)
                    members.append((w, h // 2, 0, hx, hy, blob[4:]))
            ext = "ico" if typ == 14 else "cur"
            folder = os.path.join(out, "icons" if typ == 14 else "cursors")
            os.makedirs(folder, exist_ok=True)
            hdr = struct.pack("<HHH", 0, 1 if typ == 14 else 2, count)
            off = 6 + 16 * count
            dirs = body = b""
            for (w, h, cc, a, b, blob) in members:
                dirs += struct.pack("<BBBBHHII", w & 0xFF, h & 0xFF, cc, 0, a, b, len(blob), off + len(body))
                body += blob
            path = os.path.join(folder, f"{str(name).lower()}.{ext}")
            open(path, "wb").write(hdr + dirs + body)
            if typ == 14:
                Image.open(path).save(path[:-4] + ".png")
        elif typ == 4:
            os.makedirs(os.path.join(out, "menus"), exist_ok=True)
            json.dump(parse_menu(data), open(os.path.join(out, "menus", f"{str(name).lower()}.json"), "w"), indent=2)
        elif typ == 5:
            os.makedirs(os.path.join(out, "dialogs"), exist_ok=True)
            json.dump(parse_dialog(data), open(os.path.join(out, "dialogs", f"{str(name).lower()}.json"), "w"), indent=2)

    if strings:
        json.dump(strings, open(os.path.join(out, "strings.json"), "w"), indent=2)
    json.dump(manifest, open(os.path.join(out, "manifest.json"), "w"), indent=2)
    for m in manifest:
        print(m)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
