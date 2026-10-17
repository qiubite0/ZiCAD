"""120 m2 envelope study: three bedrooms, living/dining and two bathrooms."""
from pathlib import Path
import ezdxf


def generate(path):
    d=ezdxf.new('R2010'); d.units=4; m=d.modelspace()
    for n,c in [('WALL',7),('DOOR',30),('WINDOW',4),('FURNITURE',8),('FIXTURE',3),('TEXT',7),('DIM',8),('FLOOR',8)]:
        d.layers.new(n,dxfattribs={'color':c})
    def line(a,b,l='WALL'): m.add_line(a,b,dxfattribs={'layer':l})
    def box(x,y,w,h,l='WALL'):
        p=[(x,y),(x+w,y),(x+w,y+h),(x,y+h),(x,y)]
        for a,b in zip(p,p[1:]): line(a,b,l)
    def text(s,x,y,h=150,l='TEXT'): m.add_text(s,dxfattribs={'insert':(x,y),'height':h,'layer':l})
    def wall(x1,y1,x2,y2):
        line((x1,y1),(x2,y2)); off=(120,0) if x1==x2 else (0,120)
        line((x1+off[0],y1+off[1]),(x2+off[0],y2+off[1]))
    def door(x,y,r=850,angle=0):
        if angle==0: line((x,y),(x,y+r),'DOOR')
        elif angle==90: line((x,y),(x-r,y),'DOOR')
        elif angle==180: line((x,y),(x,y-r),'DOOR')
        else: line((x,y),(x+r,y),'DOOR')
        m.add_arc((x,y),r,angle,angle+90,dxfattribs={'layer':'DOOR'})
    def window(a,b):
        line(a,b,'WINDOW')
        off=(70,0) if a[0]==b[0] else (0,70)
        line((a[0]+off[0],a[1]+off[1]),(b[0]+off[0],b[1]+off[1]),'WINDOW')
    def bed(x,y,w=1500,h=2000):
        box(x,y,w,h,'FURNITURE'); box(x,y+h-140,w,140,'FURNITURE')
        box(x+100,y+h-550,w/2-150,350,'FURNITURE');box(x+w/2+50,y+h-550,w/2-150,350,'FURNITURE')
        line((x,y+500),(x+w,y+500),'FURNITURE')
        box(x-400,y+h-450,350,450,'FURNITURE')
    def basin(x,y):
        box(x,y,650,500,'FIXTURE');m.add_ellipse((x+325,y+250),major_axis=(220,0),ratio=.65,dxfattribs={'layer':'FIXTURE'})
    def wc(x,y):
        box(x,y+350,450,200,'FIXTURE');m.add_ellipse((x+225,y+220),major_axis=(0,260),ratio=.8,dxfattribs={'layer':'FIXTURE'})
    # 12 x 10 m external footprint = 120 m2, inclusive of walls and balcony.
    # Envelope has explicit entry and window gaps.
    for a,b in [(0,1800),(4200,8200),(9200,12000)]: wall(a,0,b,0)
    for a,b in [(0,1000),(2800,10000)]: wall(0,a,0,b)
    wall(12000,0,12000,10000); wall(0,10000,12000,10000)
    window((0,1000),(0,2800)); window((1800,0),(4200,0))
    # North bedrooms; a shared hall runs beneath their doors.
    for x in [4000,8000]: wall(x,6500,x,10000)
    for a,b in [(0,3000),(3900,6900),(7800,10900),(11800,12000)]: wall(a,6500,b,6500)
    for x in [3000,6900,10900]: door(x,6500,900)
    for a,b in [(1000,3000),(5000,7000),(9000,11200)]: window((a,10000),(b,10000))
    # Living / dining west, service rooms east; service-room doors face the hall.
    for a,b in [(0,800),(1800,3800),(4700,5500)]: wall(7500,a,7500,b)
    door(7500,3800,900,90)
    for x in [7500,9800]: wall(x,5500,x,6500)
    wall(7500,3000,12000,3000)
    wall(9800,3000,9800,5500)
    for a,b in [(7500,7900),(8700,10100),(10900,12000)]: wall(a,5500,b,5500)
    door(7900,5500,800,180);door(10100,5500,800,180)
    door(8200,0,1000)
    # Kitchen and entrance foyer on southeast.
    door(7500,800,1000,90)
    for a,b in [(0,600),(1500,3000)]: wall(9800,a,9800,b)
    door(9800,600,900,90)
    window((12000,1700),(12000,2700))
    # South balcony delineation and sliding doors, entirely within envelope.
    wall(0,1600,800,1600);wall(3600,1600,4800,1600)
    wall(4800,0,4800,1600);window((800,1600),(3600,1600))

    # Bedroom furnishings.
    bed(1400,7400);box(200,7000,600,2400,'FURNITURE');box(3100,9000,700,600,'FURNITURE')
    bed(5400,7400);box(4200,6800,600,2300,'FURNITURE');box(7000,9000,750,600,'FURNITURE')
    bed(9400,7400,1800);box(8200,6800,600,2800,'FURNITURE');box(11250,9000,500,600,'FURNITURE')
    # Living sofa, coffee table and TV wall.
    box(400,2400,900,2500,'FURNITURE');box(500,2500,700,750,'FURNITURE');box(500,3350,700,650,'FURNITURE');box(500,4100,700,650,'FURNITURE')
    box(2000,3100,700,1300,'FURNITURE');box(4100,2700,350,2500,'FURNITURE');box(4150,3200,150,1500,'FURNITURE')
    # Dining: six seats, a sideboard and entrance shoe cabinet.
    box(5100,3000,1000,1800,'FURNITURE')
    for x in [4650,6200]:
        for y in [3100,3700,4300]: box(x,y,350,400,'FURNITURE')
    box(6600,1800,600,1900,'FURNITURE');box(5200,200,1800,400,'FURNITURE')
    # Kitchen L worktop, sink, hob and refrigerator.
    box(10100,200,1700,600,'FIXTURE');box(11200,800,600,2000,'FIXTURE')
    box(10150,250,650,500,'FIXTURE');basin(11200,1900)
    for x in [11350,11600]:
        for y in [1000,1300]: m.add_circle((x,y),85,dxfattribs={'layer':'FIXTURE'})
    box(10000,2100,700,700,'FIXTURE')
    # Two bathrooms with shower zones and fixtures.
    for x in [7700,10000]:
        box(x,3200,900,900,'FIXTURE');line((x,3200),(x+900,4100),'FIXTURE');line((x,4100),(x+900,3200),'FIXTURE')
        wc(x+1300,3300);basin(x+900,4700)
    # Balcony cabinet / laundry and a study nook.
    box(200,200,650,650,'FIXTURE');m.add_circle((525,525),220,dxfattribs={'layer':'FIXTURE'})
    box(1100,200,1200,450,'FURNITURE');box(5100,800,1800,500,'FURNITURE')
    for s,x,y in [('BEDROOM 2',1300,6800),('BEDROOM 3',5300,6800),('MASTER BEDROOM',8700,6800),
                  ('LIVING',2200,4900),('DINING',5300,5100),('BATH 1',7900,4400),('BATH 2',10100,4400),
                  ('KITCHEN',10100,1500),('FOYER',7800,700),('BALCONY / LAUNDRY',1000,1000),('HALL',4500,5900)]:
        text(s,x,y,160)
    # Finish hatch-like joint lines are limited to wet rooms, keeping plan legible.
    for x0,y0,w,h in [(7500,3000,4500,2500),(9800,0,2200,3000),(0,0,4800,1600)]:
        for x in range(x0+600,x0+w,600): line((x,y0),(x,y0+h),'FLOOR')
        for y in range(y0+600,y0+h,600): line((x0,y),(x0+w,y),'FLOOR')
    # Dimensions and drawing border.
    for y in [-600,10600]: line((0,y),(12000,y),'DIM')
    text('12000',5200,10700,180)
    for a,b in [(0,4000),(4000,8000),(8000,12000)]:
        line((a,10300),(b,10300),'DIM');text('4000',(a+b)/2-300,10380,140)
    line((-600,0),(-600,10000),'DIM');text('10000',-1400,4800,180)
    for x in [0,4000,8000,12000]:
        line((x,10000),(x,10900),'DIM');line((x-80,10520),(x+80,10680),'DIM')
    for y in [0,1600,6500,10000]: line((-900,y),(0,y),'DIM')
    m.add_lwpolyline([(-1800,-2200),(12800,-2200),(12800,11500),(-1800,11500)],close=True,dxfattribs={'layer':'WALL'})
    text('ZiCAD | 120 m2 APARTMENT - FURNITURE & FINISH PLAN',-1300,-1400,250)
    text('3 BED / LIVING + DINING / 2 BATH | ENVELOPE 12 x 10 m | SCHEMATIC',-1300,-1900,150)
    target=Path(path);target.parent.mkdir(parents=True,exist_ok=True);d.saveas(target)
    assert not d.audit().errors
    print(f'{target}: {len(m)} entities; DXF audit OK')


if __name__=='__main__': generate('examples/apartment-120-furnished.dxf')
