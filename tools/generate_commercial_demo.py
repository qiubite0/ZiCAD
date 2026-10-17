"""Original commercial-complex schematic base plan; dimensions are millimetres.

Run: python tools/generate_commercial_demo.py [destination.dxf]
Requires ezdxf. English drawing labels work with the current web stroke font.
"""
from pathlib import Path
import sys

import ezdxf


def generate(destination):
    doc = ezdxf.new("R2010")
    doc.units = 4
    m = doc.modelspace()
    for name, color in [("A-WALL", 7), ("A-GLAZ", 4), ("S-COLUMN", 8),
                        ("A-CORE", 3), ("A-FURN", 8), ("A-TEXT", 7),
                        ("A-GRID", 8), ("A-DIMS", 7), ("A-ATRIUM", 30)]:
        doc.layers.new(name, dxfattribs={"color": color})

    def line(a, b, layer="A-WALL"):
        m.add_line(a, b, dxfattribs={"layer": layer})

    def rect(x, y, w, h, layer="A-WALL"):
        p = [(x, y), (x+w, y), (x+w, y+h), (x, y+h), (x, y)]
        for a, b in zip(p, p[1:]):
            line(a, b, layer)

    def text(s, x, y, h=700, layer="A-TEXT"):
        m.add_text(s, dxfattribs={"height": h, "insert": (x, y), "layer": layer})

    def door(x, y, r=1400, rotation=0):
        if rotation == 0:
            line((x, y), (x, y+r))
            m.add_arc((x, y), r, 0, 90, dxfattribs={"layer": "A-WALL"})
        else:
            line((x, y), (x, y-r))
            m.add_arc((x, y), r, 270, 360, dxfattribs={"layer": "A-WALL"})

    def stairs(x, y, label):
        rect(x, y, 6500, 7200, "A-CORE")
        for n in range(13):
            yy = y+1000+n*400
            line((x+450, yy), (x+2850, yy), "A-CORE")
            line((x+3650, yy), (x+6050, yy), "A-CORE")
        line((x+3250, y+1000), (x+3250, y+6200), "A-CORE")
        text(label, x+450, y+250, 430)

    def escalator(x, y):
        for dx in [0, 2600]:
            rect(x+dx, y, 2200, 7200, "A-CORE")
            for n in range(17):
                line((x+dx+200, y+400+n*400), (x+dx+2000, y+400+n*400), "A-CORE")
            line((x+dx+1100, y+800), (x+dx+1100, y+6200), "A-ATRIUM")
        text("ESCALATORS", x-500, y-950, 500)

    # 96 x 64 m single-level study, retail ring around a central atrium.
    rect(-7500, -10500, 112000, 83500)
    rect(0, 0, 96000, 64000)
    rect(300, 300, 95400, 63400)
    # Back-of-house strips behind the north and south retail bands.
    for y in [4000, 60000]:
        line((8000, y), (88000, y))
    text("SERVICE CORRIDOR", 38500, 1600, 600)
    text("SERVICE CORRIDOR", 38500, 61500, 600)
    for i in range(10):
        x = 8000+i*8000
        for y in [4000, 48000]:
            rect(x, y, 8000, 12000)
            rect(x+250, y+250, 7500, 11500, "A-FURN")
            text(f"SHOP {'S' if y==4000 else 'N'}{i+1:02}", x+1200, y+5300, 650)
            text("96 m2", x+2200, y+4000, 470)
            front = y+12000 if y==4000 else y
            line((x+300, front), (x+6500, front), "A-GLAZ")
            door(x+6500, front, rotation=0 if y==4000 else 1)
    # Side anchor units, accessible from the east-west public loop.
    for x, prefix in [(0, "W"), (80000, "E")]:
        for i, y in enumerate([16000, 32000]):
            rect(x, y, 16000, 16000)
            text(f"ANCHOR {prefix}{i+1}", x+2600, y+8000, 800)
            text("256 m2", x+4300, y+6200, 550)
            xx = x+16000 if x==0 else x
            line((xx, y+800), (xx, y+15000), "A-GLAZ")
    # Atrium opening and continuous railing, surrounded by generous circulation.
    rect(30000, 24000, 36000, 16000, "A-ATRIUM")
    rect(30400, 24400, 35200, 15200, "A-ATRIUM")
    line((30400, 24400), (65600, 39600), "A-FURN")
    line((30400, 39600), (65600, 24400), "A-FURN")
    text("CENTRAL ATRIUM", 35500, 32500, 1200, "A-ATRIUM")
    text("OPEN TO BELOW", 38500, 30000, 750, "A-ATRIUM")
    text("PUBLIC CONCOURSE", 36500, 19300, 800)
    text("PUBLIC CONCOURSE", 36500, 44000, 800)
    escalator(22000, 28000)
    escalator(69500, 28000)
    # Four escape-stair cores and basic lifts / service rooms.
    for x in [700, 88800]:
        for y in [700, 56100]:
            stairs(x, y, "STAIR / UP")
    for x in [1000, 89000]:
        rect(x, 8500, 6000, 5800, "A-CORE")
        rect(x+600, 9100, 2200, 2600, "A-CORE")
        rect(x+3200, 9100, 2200, 2600, "A-CORE")
        text("LIFTS", x+1500, 12500, 650)
        rect(x, 49300, 6000, 5300, "A-CORE")
        text("WC / MEP", x+400, 52000, 500)
        for dx in [500, 2200, 3900]:
            rect(x+dx, 49900, 1300, 1300, "A-FURN")
    # Seating and landscape islands along circulation.
    for x in [18000, 35000, 56000, 74000]:
        for y in [17000, 45000]:
            rect(x, y, 3500, 850, "A-FURN")
            m.add_circle((x+4300, y+400), 600, dxfattribs={"layer": "A-CORE"})
    # Structural grid, with columns omitted within the atrium void.
    for ix, x in enumerate(range(0, 96001, 8000), 1):
        line((x, -1800), (x, 68000), "A-GRID")
        m.add_circle((x, 68000), 650, dxfattribs={"layer": "A-GRID"})
        text(str(ix), x-230, 67750, 480)
        if x < 96000:
            line((x, 66000), (x+8000, 66000), "A-DIMS")
            text("8000", x+2800, 66300, 420)
        for y in range(0, 64001, 8000):
            if 30000 < x < 66000 and 24000 < y < 40000:
                continue
            rect(x-350, y-350, 700, 700, "S-COLUMN")
    for i, y in enumerate(range(0, 64001, 8000)):
        line((-3000, y), (99000, y), "A-GRID")
        m.add_circle((-3500, y), 650, dxfattribs={"layer": "A-GRID"})
        text(chr(65+i), -3750, y-230, 480)
    line((0, -2500), (96000, -2500), "A-DIMS")
    text("96000", 45000, -2200, 650)
    text("ZiCAD | COMMERCIAL COMPLEX - LEVEL 02", -4500, -6200, 1400)
    text("RETAIL / ATRIUM / PUBLIC CIRCULATION / SERVICE CORES", -4500, -8400, 650)
    text("SCHEMATIC STUDY | mm | CC-02", 63000, -8400, 650)
    # Small north indicator, entirely schematic.
    line((100000, 58000), (100000, 63500), "A-DIMS")
    line((100000, 63500), (99000, 61000), "A-DIMS")
    line((100000, 63500), (101000, 61000), "A-DIMS")
    text("N", 99500, 64800, 900)
    target = Path(destination)
    target.parent.mkdir(parents=True, exist_ok=True)
    doc.saveas(target)
    audit = doc.audit()
    if audit.errors:
        raise ValueError(f"DXF audit failed: {len(audit.errors)} errors")
    print(f"{target}: {len(m)} entities; audit OK")


if __name__ == "__main__":
    generate(sys.argv[1] if len(sys.argv) > 1 else "examples/commercial-complex-demo.dxf")
