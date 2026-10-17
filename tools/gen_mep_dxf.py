# -*- coding: utf-8 -*-
"""
生成 BIM 管线综合（管综）平面图 DXF，供 CADCraft 打开。
用法: python gen_mep_dxf.py [zh|en]   (默认 zh)
  zh -> output/mep-coordination.dxf     中文标注（交付级；web 版内置无 CJK 字体显示为方框，
                                          在浩辰/AutoCAD/CADCraft 桌面版中正常）
  en -> output/mep-coordination-en.dxf  英文标注（web 演示可读）
底图为简化建筑平面（12000x8000 户型），叠加六专业管线：
  暖通风管 / 空调水 / 给水 / 排水 / 消防喷淋 / 电力桥架 / 弱电
单位 mm，出图比例 1:100。
"""
import sys
from pathlib import Path
import ezdxf

LANG = (sys.argv[1] if len(sys.argv) > 1 else "zh").lower()

S = {
 "zh": {
  "living": "LIVING / DINING 客厅", "bath": "BATH 卫生间", "bed": "BEDROOM 卧室",
  "ahu": "AHU 新风机组", "sa": "SA 新风管 SC-400x320", "cw": "空调水供回水 DN50",
  "jw": "JL 给水 DN25", "dw": "PL 排水 DN100", "fp": "ZP 消防 DN100",
  "ct": "CT 电力桥架 200x100", "td": "TD 弱电桥架 100x50",
  "well": "水暖井", "legend": "图  例",
  "lg_sa": "新风风管 SC-400x320", "lg_cw": "空调水供回水 DN50",
  "lg_jw": "给水 JL DN25", "lg_dw": "排水 PL DN100",
  "lg_fp": "消防喷淋 ZP DN100", "lg_ct": "电力桥架 CT-200x100",
  "lg_td": "弱电桥架 TD-100x50",
  "title": "BIM 管线综合平面图  ME COORDINATION PLAN",
  "sub": "比例 SCALE 1:100    单位 mm    2026-10-17    三层办公样板区",
 },
 "en": {
  "living": "LIVING / DINING", "bath": "BATH", "bed": "BEDROOM",
  "ahu": "AHU Fresh Air Unit", "sa": "SA Duct SC-400x320", "cw": "CHW Supply/Return DN50",
  "jw": "JL Domestic Water DN25", "dw": "PL Drainage DN100", "fp": "ZP Sprinkler DN100",
  "ct": "CT Cable Tray 200x100", "td": "TD ELV Tray 100x50",
  "well": "Shaft", "legend": "LEGEND",
  "lg_sa": "Fresh air duct SC-400x320", "lg_cw": "CHW pipework DN50",
  "lg_jw": "Domestic water JL DN25", "lg_dw": "Drainage PL DN100",
  "lg_fp": "Sprinkler ZP DN100", "lg_ct": "Cable tray CT-200x100",
  "lg_td": "ELV tray TD-100x50",
  "title": "BIM ME COORDINATION PLAN",
  "sub": "SCALE 1:100    unit mm    2026-10-17    Level-3 office sample area",
 },
}[LANG]

doc = ezdxf.new("R2018", setup=True)
msp = doc.modelspace()

# ---------------- 图层定义（专业分色遵循常见 CAD 惯例） ----------------
layers = [
    ("AXIS",      8,  "CENTER",     "轴线"),
    ("WALL",      8,  "CONTINUOUS", "建筑底图墙体"),
    ("HVAC-AIR",  3,  "CONTINUOUS", "暖通-新风风管"),
    ("HVAC-W",    4,  "CONTINUOUS", "暖通-空调水供回水"),
    ("PL-W",      5,  "CONTINUOUS", "给水"),
    ("PL-D",      32, "DASHED",     "污水排水"),
    ("FP",        1,  "CONTINUOUS", "消防-喷淋"),
    ("EL-C",      30, "CONTINUOUS", "电力桥架"),
    ("EL-V",      6,  "DASHED",     "弱电桥架"),
    ("EQUIP",     7,  "CONTINUOUS", "设备/末端符号"),
    ("DIM-LBL",   7,  "CONTINUOUS", "标注文字"),
    ("TITLE",     7,  "CONTINUOUS", "图框标题"),
]
for name, color, lt, desc in layers:
    doc.layers.add(name, color=color, linetype=lt)

def line(p1, p2, layer):
    msp.add_line(p1, p2, dxfattribs={"layer": layer})

def rect(x, y, w, h, layer):
    msp.add_lwpolyline([(x, y), (x + w, y), (x + w, y + h), (x, y + h)],
                       close=True, dxfattribs={"layer": layer})

def dline(x1, y1, x2, y2, gap, layer):
    """双线（表示有宽度的管/风管），gap = 管宽"""
    if abs(y2 - y1) < 1e-6:
        for dy in (-gap / 2, gap / 2):
            line((x1, y1 + dy), (x2, y1 + dy), layer)
    else:
        for dx in (-gap / 2, gap / 2):
            line((x1 + dx, y1), (x1 + dx, y2), layer)

def text(x, y, s, h=250, layer="DIM-LBL", color=None):
    att = {"layer": layer, "height": h}
    if color is not None:
        att["color"] = color
    t = msp.add_text(s, dxfattribs=att)
    t.set_placement((x, y))

# ---------------- 1. 轴网 ----------------
for i, x in enumerate([0, 7000, 12000]):
    line((x, -1500), (x, 9500), "AXIS")
    msp.add_circle((x, 9800), 500, dxfattribs={"layer": "AXIS"})
    text(x - (250 if i else 150), 9620, str(i + 1), 450, "AXIS")
for j, y in enumerate([0, 8000]):
    line((-1500, y), (13500, y), "AXIS")
    msp.add_circle((-2000, y), 500, dxfattribs={"layer": "AXIS"})
    text(-2150, y - 220, chr(ord("A") + j), 450, "AXIS")

# ---------------- 2. 建筑底图 ----------------
rect(0, 0, 12000, 8000, "WALL")
rect(200, 200, 11600, 7600, "WALL")
dline(7000, 2000, 7000, 7800, 100, "WALL")
line((6950, 2000), (6950, 7800), "WALL"); line((7050, 2000), (7050, 7800), "WALL")
dline(7050, 4500, 11800, 4500, 100, "WALL")
line((7050, 4450), (11800, 4450), "WALL"); line((7050, 4550), (11800, 4550), "WALL")
text(3000, 3800, S["living"], 300, "WALL")
text(8600, 5900, S["bath"], 300, "WALL")
text(8600, 2600, S["bed"], 300, "WALL")

# ---------------- 3. 暖通：新风风管 SC-400x320 ----------------
rect(500, 500, 900, 600, "HVAC-AIR")
text(500, 1300, S["ahu"], 220, "HVAC-AIR")
dline(1400, 1000, 6600, 1000, 400, "HVAC-AIR")
dline(6600, 1000, 6600, 5800, 400, "HVAC-AIR")
dline(6600, 5800, 11400, 5800, 400, "HVAC-AIR")
dline(8800, 1000, 8800, 2400, 400, "HVAC-AIR")
line((11380, 5600), (11380, 6000), "HVAC-AIR")
def diffuser(x, y, label):
    rect(x, y, 300, 300, "EQUIP")
    line((x, y), (x + 300, y + 300), "EQUIP")
    line((x, y + 300), (x + 300, y), "EQUIP")
    text(x + 350, y, label, 200)
diffuser(2800, 850, "CDP-300x300")
diffuser(8650, 2100, "CDP-300x300")
diffuser(11000, 5650, "CDP-300x300")
text(3200, 1500, S["sa"], 250, "HVAC-AIR")

# ---------------- 4. 暖通：空调水 DN50 ----------------
for dy in (1500, 1650):
    line((1400, dy), (6200, dy), "HVAC-W")
dline(6200, 1575, 6200, 5200, 300, "HVAC-W")
dline(6200, 5200, 11400, 5200, 300, "HVAC-W")
dline(6200, 1575, 6200, 2400, 300, "HVAC-W")
dline(6200, 2400, 9800, 2400, 300, "HVAC-W")
def fcu(x, y, label):
    rect(x, y, 400, 250, "EQUIP")
    line((x, y), (x + 400, y + 250), "EQUIP")
    line((x, y + 250), (x + 400, y), "EQUIP")
    text(x + 450, y, label, 200, "HVAC-W")
fcu(10000, 5100, "FCU-68")
fcu(9400, 2325, "FCU-50")
text(2000, 1900, S["cw"], 250, "HVAC-W")

# ---------------- 5. 给水 DN25 ----------------
rect(400, 300, 400, 400, "PL-W")
text(400, 100, S["well"], 220, "PL-W")
line((800, 500), (7300, 500), "PL-W")
line((7300, 500), (7300, 7300), "PL-W")
line((7300, 7300), (10800, 7300), "PL-W")
text(8600, 7350, S["jw"], 250, "PL-W")

# ---------------- 6. 排水 DN100 ----------------
msp.add_circle((11300, 5000), 150, dxfattribs={"layer": "PL-D"})
text(11500, 4900, "WL-1", 250, "PL-D")
line((11300, 5000), (11300, 7300), "PL-D")
line((11300, 7300), (10000, 7300), "PL-D")
line((11300, 5000), (7600, 5000), "PL-D")
msp.add_circle((8600, 5600), 100, dxfattribs={"layer": "PL-D"})
line((8600, 5600), (8600, 5030), "PL-D")
text(9200, 4650, S["dw"], 250, "PL-D")

# ---------------- 7. 消防喷淋 DN100 ----------------
dline(600, 7700, 11400, 7700, 100, "FP")
text(2000, 7750, S["fp"], 250, "FP")
def sprinkler(x, y):
    msp.add_circle((x, y), 100, dxfattribs={"layer": "EQUIP"})
    line((x - 180, y), (x + 180, y), "EQUIP")
    line((x, y - 180), (x, y + 180), "EQUIP")
for x in (2500, 5000, 11500):
    line((x, 7700), (x, 6800), "FP"); sprinkler(x, 6800)
for x in (7500, 10500):
    line((x, 7700), (x, 6200), "FP"); sprinkler(x, 6200)
for x, y in ((3000, 3000), (5500, 2500), (9000, 1000), (11000, 2000)):
    line((x, 7700), (x, y), "FP"); sprinkler(x, y)

# ---------------- 8. 电力桥架 + 配电箱 ----------------
dline(600, 4000, 11400, 4000, 200, "EL-C")
text(2000, 4100, S["ct"], 250, "EL-C")
def db(x, y, label):
    rect(x, y, 500, 350, "EQUIP")
    text(x + 550, y, label, 220, "EL-C")
db(3200, 4600, "AL-1"); line((3450, 4100), (3450, 4600), "EL-C")
db(9800, 3200, "AL-2"); line((10050, 4100), (10050, 3550), "EL-C")

# ---------------- 9. 弱电桥架 ----------------
dline(600, 3600, 7000, 3600, 100, "EL-V")
text(2000, 3250, S["td"], 250, "EL-V")

# ---------------- 10. 图例 ----------------
lx, ly = 13300, 7300
rect(lx, ly - 4600, 3100, 4600, "TITLE")
text(lx + 1100, ly - 500, S["legend"], 350, "TITLE")
legend = [
    ("HVAC-AIR", S["lg_sa"]),
    ("HVAC-W",   S["lg_cw"]),
    ("PL-W",     S["lg_jw"]),
    ("PL-D",     S["lg_dw"]),
    ("FP",       S["lg_fp"]),
    ("EL-C",     S["lg_ct"]),
    ("EL-V",     S["lg_td"]),
]
for i, (lay, name) in enumerate(legend):
    yy = ly - 1200 - i * 520
    line((lx + 200, yy + 100), (lx + 1000, yy + 100), lay)
    text(lx + 1200, yy, name, 260)

# ---------------- 11. 图框 + 标题栏 ----------------
rect(-3000, -2500, 20000, 13000, "TITLE")
text(-3000, -2350, S["title"], 400, "TITLE")
text(-3000, -1800, S["sub"], 300, "TITLE")

out = Path(__file__).resolve().parents[1] / "apps/cadcraft-web/dev-host" / ("mep-coordination.dxf" if LANG == "zh" else "mep-coordination-en.dxf")

doc.saveas(out)
print("saved:", out)
