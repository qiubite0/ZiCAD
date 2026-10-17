"""Generate an original, schematic cleanroom coordination DXF for UI demos."""
from pathlib import Path
import sys

import ezdxf


def generate(destination):
    doc = ezdxf.new("R2010")
    doc.units = 4
    space = doc.modelspace()
    for name, color in [("WALL", 7), ("EQUIPMENT", 8), ("SUPPLY", 4),
                        ("RETURN", 3), ("PROCESS", 30), ("TRAY", 6),
                        ("TEXT", 7), ("AXIS", 8)]:
        doc.layers.new(name, dxfattribs={"color": color})

    def line(a, b, layer="WALL"):
        space.add_line(a, b, dxfattribs={"layer": layer})

    def box(x, y, w, h, layer="WALL"):
        points = [(x, y), (x+w, y), (x+w, y+h), (x, y+h), (x, y)]
        for a, b in zip(points, points[1:]):
            line(a, b, layer)

    def text(value, x, y, size=180, layer="TEXT"):
        space.add_text(value, dxfattribs={"height": size, "insert": (x, y), "layer": layer})

    box(-1500, -1700, 28000, 20500)
    box(0, 0, 24000, 16000)
    box(150, 150, 23700, 15700)
    for x in [6000, 12000, 18000]:
        line((x, 12000), (x, 16000))
        line((x+100, 12000), (x+100, 16000))
    for a, b in [(0, 6000), (7000, 12000), (13000, 18000), (19000, 24000)]:
        line((a, 12000), (b, 12000))
        line((a, 12100), (b, 12100))
    for x in [6000, 12000, 18000]:
        line((x, 12000), (x, 13000))
        space.add_arc((x, 12000), 1000, 0, 90, dxfattribs={"layer": "WALL"})
    for name, x in [("MATERIAL AIRLOCK", 700), ("GOWNING", 7400),
                    ("AIR SHOWER", 13000), ("UTILITY ROOM", 18800)]:
        text(name, x, 14200, 220)
    text("CLEAN PRODUCTION HALL", 6500, 10400, 350)
    for x in [2200, 6200, 10200, 14200, 18200]:
        for y in [3100, 6500]:
            box(x, y, 2200, 1600, "EQUIPMENT")
            box(x+150, y+150, 1900, 1300, "EQUIPMENT")
            text("PROCESS", x+350, y+650, 160)
    for y in [9300, 9650]:
        line((1400, y), (22200, y), "SUPPLY")
    text("SA 800x350", 1700, 9900, 200, "SUPPLY")
    for x in [3300, 7300, 11300, 15300, 19300]:
        line((x-100, 9300), (x-100, 8400), "SUPPLY")
        line((x+100, 9300), (x+100, 8400), "SUPPLY")
        box(x-450, 7800, 900, 600, "SUPPLY")
        line((x-450, 7800), (x+450, 8400), "SUPPLY")
        line((x-450, 8400), (x+450, 7800), "SUPPLY")
    for y in [1200, 1450]:
        line((1400, y), (22000, y), "RETURN")
    text("RA 600x250", 1700, 1600, 200, "RETURN")
    for y, layer, label in [(2300, "PROCESS", "PROCESS DN50"),
                             (11000, "TRAY", "CABLE TRAY 300x100")]:
        line((1000, y), (22500, y), layer)
        text(label, 1200, y+220, 190, layer)
    for x in [3300, 7300, 11300, 15300, 19300]:
        line((x, 2300), (x, 3100), "PROCESS")
        space.add_circle((x, 2600), 90, dxfattribs={"layer": "PROCESS"})
    for i, x in enumerate(range(0, 24001, 6000), 1):
        line((x, -500), (x, 17500), "AXIS")
        space.add_circle((x, 17500), 260, dxfattribs={"layer": "AXIS"})
        text(str(i), x-70, 17400, 200)
        if x < 24000:
            line((x, 16700), (x+6000, 16700), "TEXT")
            text("6000", x+2500, 16800, 180)
    line((0, -600), (24000, -600), "TEXT")
    text("24000", 11000, -500, 200)
    text("ZiCAD | CLEANROOM MEP COORDINATION", -900, -1250, 350)
    text("SCHEMATIC DEMO   /   mm   /   CR-01", 15500, -1250, 220)
    target = Path(destination)
    target.parent.mkdir(parents=True, exist_ok=True)
    doc.saveas(target)
    print(target)


if __name__ == "__main__":
    generate(sys.argv[1] if len(sys.argv) > 1 else "examples/cleanroom-demo.dxf")
