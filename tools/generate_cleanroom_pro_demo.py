"""Detailed original cleanroom coordination study, not construction documents."""
from pathlib import Path
import sys

import ezdxf


def generate(destination):
    d = ezdxf.new("R2010")
    d.units = 4
    m = d.modelspace()
    layers = {"WALL": 7, "GLAZING": 4, "EQUIP": 8, "FFU": 4,
              "SA": 4, "RA": 3, "EXHAUST": 1, "PROCESS": 30,
              "TRAY": 6, "PERSONNEL": 2, "MATERIAL": 5,
              "TEXT": 7, "GRID": 8, "DIM": 7}
    for name, color in layers.items():
        d.layers.new(name, dxfattribs={"color": color})

    def ln(a, b, layer="WALL"):
        m.add_line(a, b, dxfattribs={"layer": layer})

    def box(x, y, w, h, layer="WALL"):
        p = [(x,y),(x+w,y),(x+w,y+h),(x,y+h),(x,y)]
        for a,b in zip(p,p[1:]):
            ln(a,b,layer)

    def tx(s,x,y,h=220,layer="TEXT"):
        m.add_text(s,dxfattribs={"insert":(x,y),"height":h,"layer":layer})

    def wall(a,b):
        ln(a,b)
        offset = (120,0) if a[0]==b[0] else (0,120)
        ln((a[0]+offset[0],a[1]+offset[1]),(b[0]+offset[0],b[1]+offset[1]))

    def door(x,y,r=1000,down=False):
        ln((x,y),(x,y-r if down else y+r))
        m.add_arc((x,y),r,270 if down else 0,360 if down else 90,
                  dxfattribs={"layer":"WALL"})

    def route(points,layer,width=0):
        for a,b in zip(points,points[1:]):
            ln(a,b,layer)
            if width:
                off=(width,0) if a[0]==b[0] else (0,width)
                ln((a[0]+off[0],a[1]+off[1]),(b[0]+off[0],b[1]+off[1]),layer)

    def arrow(a,b,layer):
        ln(a,b,layer)
        dx,dy=b[0]-a[0],b[1]-a[1]
        length=(dx*dx+dy*dy)**0.5
        if length:
            ux,uy=dx/length,dy/length
            for sign in [-1,1]:
                ln(b,(b[0]-450*ux+sign*180*uy,b[1]-450*uy-sign*180*ux),layer)

    box(-2500,-4500,45500,31500)
    box(0,0,32000,24000)
    box(150,150,31700,23700)
    # North service suite and production frontages with actual door gaps.
    for x in [6000,12000,18000,24000]:
        wall((x,19000),(x,24000))
    for a,b in [(0,4500),(5500,10500),(11500,16500),(17500,22500),(23500,32000)]:
        wall((a,19000),(b,19000))
    for x in [4500,10500,16500,22500]:
        door(x,19000)
    labels=[("01 PRE-GOWN",600),("02 FINAL GOWN",6500),
            ("03 AIR SHOWER",12500),("04 MAT. AIRLOCK",18300),
            ("05 AHU / UTILITIES",24500)]
    for s,x in labels:
        tx(s,x,22200,250)
    for x in [6500,8100,9700]:
        box(x,22800,1100,700,"EQUIP")
    for y in [19900,20700,21500]:
        box(12800,y,550,500,"EQUIP")
        box(16700,y,550,500,"EQUIP")
    box(19000,20200,2000,1600,"EQUIP")
    tx("CART",19300,20700,200)
    box(24900,20000,5800,1600,"EQUIP")
    for x in [25800,27400,29000]:
        ln((x,20000),(x,21600),"EQUIP")
    tx("AHU-01",26500,20500,250)
    # South suite opens to a controlled clean corridor, isolated from production.
    wall((0,4200),(32000,4200))
    for x in [6000,12000,20000,26000]:
        wall((x,0),(x,4200))
    for a,b in [(0,4500),(5500,10500),(11500,18500),(19500,24500),(25500,32000)]:
        wall((a,7000),(b,7000))
    for x in [4500,10500,18500,24500]:
        door(x,7000,down=True)
    for s,x in [("06 WASH",600),("07 QC LAB",6500),("08 PACKAGING",13000),
                ("09 STORAGE",20500),("10 MEP",27000)]:
        tx(s,x,2100,260)
    tx("CONTROLLED CLEAN CORRIDOR",9500,5400,300)
    tx("11 CLEAN PROCESS HALL",10500,16600,420)
    # Partitioned process cells and glazing; openings face the central aisle.
    for x in [10500,21000]:
        wall((x,7000),(x,10500))
        wall((x,13500),(x,19000))
        ln((x,14000),(x,18000),"GLAZING")
    for s,x in [("CELL A / PREP",1700),("CELL B / PROCESS",12000),("CELL C / FINISH",22500)]:
        tx(s,x,15100,280)
    tx("MAINTENANCE / TRANSFER AISLE",10500,11800,240)
    for idx,x in enumerate([1500,5200,12000,15700,22500,26200],1):
        for j,y in enumerate([8000,13500]):
            box(x,y,2600,1500,"EQUIP")
            box(x+180,y+180,2240,1140,"EQUIP")
            box(x+800,y+450,900,600,"EQUIP")
            tx(f"EQ-{idx:02}{j+1}",x+450,y+600,210)
    # Material pass boxes with smaller plan detail.
    for x in [17500,23500]:
        box(x,6850,1100,450,"EQUIP")
        tx("PB",x+200,7450,180)
    # Ceiling FFUs on two regular rows, drawn distinctly from process equipment.
    for row,y in enumerate([10000,17000],1):
        for col,x in enumerate([2500,7000,13000,17500,23500,28000],1):
            box(x,y,1200,600,"FFU")
            ln((x,y),(x+1200,y+600),"FFU")
            ln((x+1200,y),(x,y+600),"FFU")
            tx(f"FFU-{row}{col}",x-50,y+800,160,"FFU")
    # Supply headers, branches, low returns, local exhaust and utility circuits.
    route([(30100,20000),(30100,18200),(1800,18200)],"SA",250)
    route([(30100,18200),(30100,10800),(1800,10800)],"SA",250)
    for y in [10000,17000]:
        for x in [2500,7000,13000,17500,23500,28000]:
            route([(x+600,y+600),(x+600,10800 if y==10000 else 18200)],"SA",100)
    tx("SA MAIN 800x350",18500,18600,230,"SA")
    route([(800,7500),(800,15800),(31000,15800),(31000,20700)],"RA",180)
    tx("RA MAIN / LOW RETURN GRILLES",4500,16050,200,"RA")
    for x in [400,10700,21200,30700]:
        box(x,7500,300,900,"RA")
        for k in range(5):
            ln((x,7600+k*150),(x+300,7600+k*150),"RA")
    route([(2000,12800),(29400,12800),(29400,23200)],"EXHAUST",100)
    tx("LOCAL EXHAUST 400x250",12000,13050,210,"EXHAUST")
    for x in [2800,6500,13300,17000,23800,27500]:
        ln((x,12800),(x,13500),"EXHAUST")
    for n,label in enumerate(["PCW DN40","CDA DN25","N2 DN20"]):
        y=8700+n*250
        route([(31000,3000),(31000+n*200,y),(1800,y)],"PROCESS")
        tx(label,19000,y+70,150,"PROCESS")
    route([(31500,2000),(31500,14500),(1200,14500)],"TRAY",150)
    tx("CABLE TRAY 300x100",1500,14750,200,"TRAY")
    # Separate personnel and material circulation, avoiding illustrative overlap.
    arrow((1500,20000),(7500,20000),"PERSONNEL")
    arrow((7500,20000),(14000,20000),"PERSONNEL")
    arrow((14000,20000),(14000,15500),"PERSONNEL")
    arrow((20500,20500),(20500,17500),"MATERIAL")
    arrow((20500,17500),(24500,17500),"MATERIAL")
    # Notes and legend are outside the plan, readable without covering geometry.
    tx("SYSTEM LEGEND",34000,22500,500)
    legend=[("FFU","CEILING FFU"),("SA","SUPPLY AIR"),("RA","RETURN AIR"),
            ("EXHAUST","LOCAL EXHAUST"),("PROCESS","PROCESS UTILITIES"),
            ("TRAY","CABLE TRAY"),("PERSONNEL","PERSONNEL FLOW"),
            ("MATERIAL","MATERIAL FLOW")]
    for i,(layer,label) in enumerate(legend):
        y=21200-i*1100
        ln((34000,y),(35500,y),layer)
        tx(label,36000,y-100,230)
    for i,s in enumerate(["DESIGN NOTES", "1. SCHEMATIC COORDINATION STUDY",
                           "2. NO CLEANLINESS CLASS ASSIGNED",
                           "3. AIRFLOW / PRESSURE TO BE ENGINEERED",
                           "4. VERIFY ACCESS AND MAINTENANCE",
                           "5. FFU SYMBOLS ARE CEILING OVERLAY"]):
        tx(s,34000,10800-i*900,210 if i else 350)
    # Drawing grid and overall / chain dimensions.
    for i,x in enumerate([0,6000,12000,18000,24000,32000],1):
        ln((x,-700),(x,25000),"GRID")
        m.add_circle((x,25000),220,dxfattribs={"layer":"GRID"})
        tx(str(i),x-60,24920,160)
        box(x-150,150,300,300,"GRID")
    for i,y in enumerate([0,7000,12000,19000,24000]):
        ln((-1000,y),(32500,y),"GRID")
        m.add_circle((-1000,y),220,dxfattribs={"layer":"GRID"})
        tx(chr(65+i),-1060,y-80,160)
    ln((0,-900),(32000,-900),"DIM")
    tx("32000",14500,-800,220)
    for a,b in zip([0,6000,12000,18000,24000],[6000,12000,18000,24000,32000]):
        ln((a,24400),(b,24400),"DIM")
        tx(str(b-a),(a+b)/2-500,24500,170)
    ln((0,-1700),(40500,-1700),"DIM")
    tx("ZiCAD | CLEANROOM - PROCESS & MEP COORDINATION",-1000,-2700,500)
    tx("ARCHITECTURE / FFU / HVAC / UTILITIES / FLOW STUDY",-1000,-3800,280)
    tx("CR-02  |  mm  |  CONCEPT STUDY",29000,-3800,240)
    target=Path(destination)
    target.parent.mkdir(parents=True,exist_ok=True)
    d.saveas(target)
    if d.audit().errors:
        raise ValueError("DXF audit failed")
    print(f"{target}: {len(m)} entities; audit OK")


if __name__ == "__main__":
    generate(sys.argv[1] if len(sys.argv)>1 else "examples/cleanroom-pro-demo.dxf")
