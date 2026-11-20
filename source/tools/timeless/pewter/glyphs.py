import math
def P(pts): return "M"+" L".join(f"{x:.2f} {y:.2f}" for x,y in pts)+" Z"
def circ(cx,cy,r):
    return f"M{cx-r:.2f} {cy:.2f} A{r} {r} 0 1 0 {cx+r:.2f} {cy:.2f} A{r} {r} 0 1 0 {cx-r:.2f} {cy:.2f} Z"
def rot(pts,a,cx,cy):
    c,s=math.cos(a),math.sin(a)
    return [(cx+x*c-y*s, cy+x*s+y*c) for x,y in pts]
def star(cx,cy,ro,ri,n=4,off=-90):
    pts=[]
    for i in range(2*n):
        r=ro if i%2==0 else ri; a=math.radians(off+i*180/n)
        pts.append((cx+r*math.cos(a),cy+r*math.sin(a)))
    return P(pts)
def sword(cx,cy,ang):
    a=math.radians(ang); out=[]
    blade=[(0,-25),(3,-20),(3,8),(-3,8),(-3,-20)]
    guard=[(-11,9.6),(-9,7.6),(9,7.6),(11,9.6),(9,11.6),(-9,11.6)]
    grip=[(-1.8,11.6),(1.8,11.6),(1.8,19),(-1.8,19)]
    out.append(("shape",P(rot(grip,a,cx,cy))))
    px,py=rot([(0,21.6)],a,cx,cy)[0]; out.append(("shape",circ(px,py,2.8)))
    out.append(("shape",P(rot(blade,a,cx,cy))))
    out.append(("shape",P(rot(guard,a,cx,cy))))
    (x1,y1),(x2,y2)=rot([(0,-18),(0,5)],a,cx,cy); out.append(("line",f"M{x1:.2f} {y1:.2f} L{x2:.2f} {y2:.2f}"))
    return out
def gear():
    pts=[]
    for i in range(8):
        a=i*45
        for da,r in ((-16,17),(-9,23.5),(9,23.5),(16,17)):
            t=math.radians(a+da-90); pts.append((32+r*math.cos(t),32+r*math.sin(t)))
    return [("shape",P(pts)+" "+circ(32,32,7)),("line",circ(32,32,12))]
def compass():
    ring=circ(32,32,21)+" "+circ(32,32,18)
    g=[("shape",ring),("shape",star(32,32,17,4.2,4,-45)),("shape",star(32,32,26.5,5.2,4,-90))]
    for k in range(4):
        t=math.radians(-90+k*90); g.append(("line",f"M32 32 L{32+24*math.cos(t):.2f} {32+24*math.sin(t):.2f}"))
    g.append(("dot",circ(32,32,2.2)))
    return g
def scallop():
    H=(32,52); n=7; th=[math.radians(165-i*(150/(n-1))) for i in range(n)]
    pts=[(32+24*math.cos(t),31-22*math.sin(t)) for t in th]
    d=f"M{H[0]} {H[1]} L{pts[0][0]:.2f} {pts[0][1]:.2f}"
    for i in range(n-1):
        tm=(th[i]+th[i+1])/2; cx,cy=32+27.5*math.cos(tm),31-25.5*math.sin(tm)
        d+=f" Q{cx:.2f} {cy:.2f} {pts[i+1][0]:.2f} {pts[i+1][1]:.2f}"
    d+=" Z"
    g=[("shape",P([(32,52),(21,53),(23,46),(30,47)])),("shape",P([(32,52),(43,53),(41,46),(34,47)])),("shape",d)]
    for x,y in pts[1:-1]:
        g.append(("line",f"M32 49 L{32+(x-32)*0.9:.2f} {49+(y-49)*0.9:.2f}"))
    g.append(("shape",circ(47,48,5))); g.append(("dot",circ(45.6,46.4,1.3)))
    return g

G={}
G["Chat"]=[("shape","M30 14 C42.5 14 51 21.5 51 30 C51 38.5 42.5 46 30 46 C27 46 24.4 45.6 22 44.8 L12.5 51 L15.2 41.6 C11.5 38.6 9 34.6 9 30 C9 21.5 17.5 14 30 14 Z"),
  ("dot",circ(21.5,30,2.4)),("dot",circ(30,30,2.4)),("dot",circ(38.5,30,2.4)),("shape",star(50,13,7,2,4))]
G["Magic"]=[("shape","M32 25 C26 21 17 20 8 22 L8 49 C17 47 26 47.5 32 51 Z"),("shape","M32 25 C38 21 47 20 56 22 L56 49 C47 47 38 47.5 32 51 Z"),
  ("line","M13 29 C18 28 23 28.4 28 30 M13 35 C18 34 23 34.4 28 36 M13 41 C18 40 23 40.4 28 42 M36 30 C41 28.4 46 28 51 29 M36 36 C41 34.4 46 34 51 35 M36 42 C41 40.4 46 40 51 41"),
  ("shape",star(32,11.5,8,2.3,4))]
G["Abilities"]=[("line","M32 4 L32 10 M20 7 L23.5 12.5 M44 7 L40.5 12.5"),
  ("shape","M23 45 L45 45 L47.5 57 L20.5 57 Z"),("line","M22 51 L46 51"),
  ("shape","M17 24 C17 19.5 20 17 24 17 L45 17 C48.5 17 51 19.5 51 23 L51 35 C51 41 47 45 41 45 L27 45 C21 45 17 41 17 35 Z"),
  ("line","M25.5 17.5 L25.5 28 M34 17.5 L34 28.6 M42.5 17.5 L42.5 28 M17.5 28 C28 29.6 40 29.6 50.5 28"),
  ("shape","M17 31 C23 30 30 31 33.5 34.5 C31 38.5 24 39.6 17 38 Z")]
G["Trust"]=[("shape","M19 8 L45 8 C46.7 8 48 9.3 48 11 L48 53 C48 54.7 46.7 56 45 56 L19 56 C17.3 56 16 54.7 16 53 L16 11 C16 9.3 17.3 8 19 8 Z"),
  ("line","M21.5 50.5 L21.5 24 C21.5 15.5 42.5 15.5 42.5 24 L42.5 50.5 Z"),
  ("shape","M24 47 C24 39.5 27.5 36.5 32 36.5 C36.5 36.5 40 39.5 40 47 Z"),("shape",circ(32,28.5,5.5)),
  ("dot",circ(19.5,11.5,1.3)),("dot",circ(44.5,11.5,1.3)),("dot",circ(19.5,52.5,1.3)),("dot",circ(44.5,52.5,1.3))]
G["Items"]=[("line","M42 21 C50 17 53 11 48 9.5"),
  ("shape","M22.5 23 C14 30.5 10 38.5 12 46.5 C14 54 22 56.5 32 56.5 C42 56.5 50 54 52 46.5 C54 38.5 50 30.5 41.5 23 Z"),
  ("shape","M24 22 C21.5 15 25 10 28 12.5 C30 8 34 8 36 12.5 C39 10 42.5 15 40 22 Z"),
  ("shape","M20 20 L44 20 L42.5 25.5 L21.5 25.5 Z"),("line","M23 37 C29 35.5 35 35.5 41 37"),("dot",circ(32,43,2.2))]
G["Attack"]=sword(32,33,-42)+sword(32,33,42)
G["Check"]=[("shape",P([(36.5,40.5),(40.5,36.5),(56,52),(52,56)])),("shape",circ(27,27,15.5)+" "+circ(27,27,11.5)),
  ("line","M19 23.5 C20.5 19.5 24 17 28 16.8")]
G["Trade"]=[("shape","M22 50 L42 50 C43 52 44 54 45.5 55.5 L18.5 55.5 C20 54 21 52 22 50 Z"),("shape",P([(30.5,21),(33.5,21),(34.5,50),(29.5,50)])),
  ("line","M14 21 L7 38 M14 21 L21 38 M50 21 L43 38 M50 21 L57 38"),
  ("shape","M10 17.5 L54 17.5 L54 21.5 L10 21.5 Z"),("shape",circ(32,14.5,3.2)),
  ("shape","M4.5 38 L23.5 38 C22.5 44.5 5.5 44.5 4.5 38 Z"),("shape","M40.5 38 L59.5 38 C58.5 44.5 41.5 44.5 40.5 38 Z")]
G["Equipment"]=[("shape","M32 7 L53 13.5 C53 34 46.5 48 32 57.5 C17.5 48 11 34 11 13.5 Z"),
  ("line","M32 12.5 L47.5 17.4 C47.5 33 42.5 44 32 51.4 C21.5 44 16.5 33 16.5 17.4 Z"),("shape",star(32,31,10,3,4))]
G["Map"]=compass()
G["Linkshell"]=scallop()
G["Party"]=[("shape","M28 46 C28 35 33 30.5 40.5 30.5 C48 30.5 53.5 35 53.5 46 Z"),("shape",circ(40.5,20.5,6.8)),
  ("shape","M10 55 C10 43 16 37.5 25 37.5 C34 37.5 40 43 40 55 Z"),("shape",circ(25,27,8))]
G["Key Items"]=[("g","rotate(-40 32 32)",[("shape",circ(15,32,10)+" "+circ(15,32,4.5)),
  ("shape",P([(47,33.5),(47,43),(51,43),(51,39),(55,39),(55,33.5)])),("shape",P([(25,30.6),(56,30.6),(56,33.6),(25,33.6)])),
  ("shape",P([(23.5,28.5),(28,28.5),(28,35.6),(23.5,35.6)])),("dot",circ(15,22.6,1.6)),("dot",circ(15,41.4,1.6)),("dot",circ(5.6,32,1.6))])]
G["Journal"]=[("shape",P([(17,14),(47,14),(47,50),(17,50)])),("line","M22 22.5 H42 M22 28 H42 M22 33.5 H37 M22 39 H33"),
  ("line","M38.5 44 L36.5 49 M43.5 44 L45.5 49"),("shape",circ(41,40.5,4.5)),
  ("shape","M14.5 8.5 L49.5 8.5 C51.5 8.5 53 10 53 12 C53 14 51.5 15.5 49.5 15.5 L14.5 15.5 C12.5 15.5 11 14 11 12 C11 10 12.5 8.5 14.5 8.5 Z"),
  ("shape","M14.5 48.5 L49.5 48.5 C51.5 48.5 53 50 53 52 C53 54 51.5 55.5 49.5 55.5 L14.5 55.5 C12.5 55.5 11 54 11 52 C11 50 12.5 48.5 14.5 48.5 Z")]
G["Mog House"]=[("shape",P([(41.5,10),(47.5,10),(47.5,23),(41.5,23)])),
  ("shape",P([(32,8),(56,28),(51,28),(51,55),(13,55),(13,28),(8,28)])),("line","M17 28 L47 28"),
  ("shape","M26.5 55 L26.5 44 C26.5 37.5 37.5 37.5 37.5 44 L37.5 55 Z"),("shape",circ(32,28.5,4.6)+" "+circ(32,28.5,2.2)),("dot",circ(35,48.5,1.2))]
G["Config"]=gear()
MENU=["Chat","Magic","Abilities","Trust","Items"]
SHEET=["Chat","Magic","Abilities","Trust","Items","Attack","Check","Trade","Equipment","Map","Linkshell","Party","Key Items","Journal","Mog House","Config"]

# --- Round 2 ---
def _ell(cx,cy,rx,ry,rot,n=72,s=1.0,tx=0,ty=0):
    a=math.radians(rot); pts=[]
    for k in range(n):
        t=2*math.pi*k/n; x,y=rx*math.cos(t),ry*math.sin(t)
        pts.append((tx+s*(cx+x*math.cos(a)-y*math.sin(a)), ty+s*(cy+x*math.sin(a)+y*math.cos(a))))
    return pts
def _sc(d_pts,s,tx,ty): return [(tx+s*x,ty+s*y) for x,y in d_pts]
def linkpearl():
    s,tx,ty=1.86,2.0,3.2
    def m(x,y): return f"{tx+s*x:.2f} {ty+s*y:.2f}"
    upper=f"M{m(20.8,5.2)} C{m(23.6,2.4)} {m(28.2,2.0)} {m(30.4,3.9)} C{m(31.6,5.1)} {m(31.1,7.3)} {m(29.6,7.6)} C{m(29.1,6.4)} {m(27.9,6.0)} {m(26.4,6.9)} C{m(25.2,7.6)} {m(24.0,8.4)} {m(22.8,9.2)} Z"
    lower=f"M{m(27.0,12.4)} C{m(29.0,11.8)} {m(31.2,13.0)} {m(31.0,15.0)} C{m(30.8,16.9)} {m(29.2,17.9)} {m(27.6,17.4)} Z"
    body=P(_ell(15.32,15.6,13.73,12.82,-16.7,s=s,tx=tx,ty=ty))
    hx,hy=tx+s*15.0,ty+s*15.8; w,h=6.2,10.5
    glass=(f"M{hx-w:.2f} {hy-h:.2f} C{hx-w:.2f} {hy-h*0.4:.2f} {hx-0.8:.2f} {hy-h*0.18:.2f} {hx-0.4:.2f} {hy:.2f} C{hx-0.8:.2f} {hy+h*0.18:.2f} {hx-w:.2f} {hy+h*0.4:.2f} {hx-w:.2f} {hy+h:.2f} "
           f"M{hx+w:.2f} {hy-h:.2f} C{hx+w:.2f} {hy-h*0.4:.2f} {hx+0.8:.2f} {hy-h*0.18:.2f} {hx+0.4:.2f} {hy:.2f} C{hx+0.8:.2f} {hy+h*0.18:.2f} {hx+w:.2f} {hy+h*0.4:.2f} {hx+w:.2f} {hy+h:.2f}")
    caps=P([(hx-w-1.6,hy-h-2.2),(hx+w+1.6,hy-h-2.2),(hx+w+1.6,hy-h),(hx-w-1.6,hy-h)])+" "+P([(hx-w-1.6,hy+h),(hx+w+1.6,hy+h),(hx+w+1.6,hy+h+2.2),(hx-w-1.6,hy+h+2.2)])
    sand=f"M{hx-4.6:.2f} {hy+h-0.4:.2f} C{hx-2.5:.2f} {hy+h-4.2:.2f} {hx+2.5:.2f} {hy+h-4.2:.2f} {hx+4.6:.2f} {hy+h-0.4:.2f} Z"
    spec=f"M{m(5.6,13.0)} C{m(6.0,9.0)} {m(8.6,6.2)} {m(12.0,5.0)}"
    return [("shape",upper),("shape",lower),("shape",body),("line",spec),("dot",circ(tx+s*7.4,ty+s*8.6,1.5)),
            ("line",glass),("dot",caps),("dot",sand)]
def companion():
    g=[]
    for side in (-1,1):
        stem=f"M{32+side*3:.2f} 57 C{32+side*15:.2f} 56 {32+side*23:.2f} 47 {32+side*23.5:.2f} 33 C{32+side*23.6:.2f} 27 {32+side*22:.2f} 22 {32+side*19.5:.2f} 18"
        g.append(("line",stem))
        # leaves along the stem
        for t,(x,y) in enumerate([(14.0,54.5),(20.6,48.6),(23.8,41.0),(24.8,33.0),(24.0,25.4),(21.6,19.0)]):
            px=32+side*(x-32+0) if False else (32-(32-x)*1 if side<0 else 32+(32-x))
            ang=math.radians([200,225,250,268,285,300][t]) if side<0 else math.radians([-20,-45,-70,-88,-105,-120][t])
            L,W=5.2,2.1; cx,cy=px+math.cos(ang)*2.6, y+math.sin(ang)*2.6
            tip=(cx+math.cos(ang)*L/2, cy+math.sin(ang)*L/2); bas=(cx-math.cos(ang)*L/2, cy-math.sin(ang)*L/2)
            nx,ny=-math.sin(ang)*W,math.cos(ang)*W
            d=(f"M{bas[0]:.2f} {bas[1]:.2f} Q{cx+nx:.2f} {cy+ny:.2f} {tip[0]:.2f} {tip[1]:.2f} Q{cx-nx:.2f} {cy-ny:.2f} {bas[0]:.2f} {bas[1]:.2f} Z")
            g.append(("shape",d))
    g.append(("shape","M19.5 53 C19.5 41.5 25 36 32 36 C39 36 44.5 41.5 44.5 53 Z"))
    g.append(("shape",circ(32,25.5,8)))
    g.append(("line","M27 45 L32 50.5 L37 45"))
    g.append(("shape",star(32,9.5,6.5,1.9,4)))
    return g
G["Companion"]=companion()
G["Linkshell"]=linkpearl()
MENU=["Chat","Magic","Abilities","Companion","Items"]
SHEET=["Chat","Magic","Abilities","Companion","Items","Attack","Check","Trade","Equipment","Map","Linkshell","Party","Key Items","Journal","Mog House","Config"]

# --- Round 2b: one-silhouette linkpearl, laurel companion ---
def linkpearl2():
    s,tx,ty=1.86,2.2,3.4
    m=lambda x,y: f"{tx+s*x:.2f} {ty+s*y:.2f}"
    pts=_ell(15.32,15.6,13.73,12.82,-16.7,n=144)
    keep=[(x,y) for x,y in pts if not (x>24.6 and y<18.2)]
    # rotate list so it starts right after the excluded arc (bottom-right) and ends at the top-right
    idx=[i for i,(x,y) in enumerate(pts) if (x>24.6 and y<18.2)]
    start=(idx[-1]+1)%len(pts); order=[pts[(start+k)%len(pts)] for k in range(len(pts))]
    order=[p for p in order if not (p[0]>24.6 and p[1]<18.2)]
    d="M"+" L".join(m(x,y) for x,y in order)
    d+=(f" L{m(24.6,3.9)} C{m(26.4,3.2)} {m(28.6,2.7)} {m(29.9,3.0)} C{m(31.4,3.4)} {m(31.6,5.6)} {m(30.3,7.6)}"
        f" C{m(30.0,6.4)} {m(29.2,5.9)} {m(28.2,6.1)} C{m(27.6,6.3)} {m(27.3,6.9)} {m(27.3,7.6)} L{m(27.3,12.4)}"
        f" C{m(29.0,12.2)} {m(30.9,13.0)} {m(31.0,15.0)} C{m(31.0,16.8)} {m(29.6,17.8)} {m(27.6,18.2)}")
    first=order[0]; d+=f" L{m(*first)} Z"
    hx,hy=tx+s*14.6,ty+s*16.0; w,h=6.0,10.0
    glass=(f"M{hx-w:.2f} {hy-h:.2f} C{hx-w:.2f} {hy-h*0.4:.2f} {hx-0.8:.2f} {hy-h*0.18:.2f} {hx-0.4:.2f} {hy:.2f} C{hx-0.8:.2f} {hy+h*0.18:.2f} {hx-w:.2f} {hy+h*0.4:.2f} {hx-w:.2f} {hy+h:.2f} "
           f"M{hx+w:.2f} {hy-h:.2f} C{hx+w:.2f} {hy-h*0.4:.2f} {hx+0.8:.2f} {hy-h*0.18:.2f} {hx+0.4:.2f} {hy:.2f} C{hx+0.8:.2f} {hy+h*0.18:.2f} {hx+w:.2f} {hy+h*0.4:.2f} {hx+w:.2f} {hy+h:.2f}")
    caps=P([(hx-w-1.8,hy-h-2.2),(hx+w+1.8,hy-h-2.2),(hx+w+1.8,hy-h),(hx-w-1.8,hy-h)])+" "+P([(hx-w-1.8,hy+h),(hx+w+1.8,hy+h),(hx+w+1.8,hy+h+2.2),(hx-w-1.8,hy+h+2.2)])
    sand=f"M{hx-4.4:.2f} {hy+h-0.3:.2f} C{hx-2.4:.2f} {hy+h-4.0:.2f} {hx+2.4:.2f} {hy+h-4.0:.2f} {hx+4.4:.2f} {hy+h-0.3:.2f} Z"
    spec=f"M{m(4.6,15.0)} C{m(4.6,10.0)} {m(7.6,6.0)} {m(11.6,4.6)}"
    return [("shape",d),("line",spec),("dot",circ(tx+s*7.2,ty+s*8.4,1.5)),("line",glass),("dot",caps),("dot",sand),
            ("line",circ(tx+s*29.0,ty+s*15.0,1.1))]
def _bez(p0,p1,p2,p3,t):
    u=1-t
    x=u**3*p0[0]+3*u*u*t*p1[0]+3*u*t*t*p2[0]+t**3*p3[0]; y=u**3*p0[1]+3*u*u*t*p1[1]+3*u*t*t*p2[1]+t**3*p3[1]
    dx=3*u*u*(p1[0]-p0[0])+6*u*t*(p2[0]-p1[0])+3*t*t*(p3[0]-p2[0]); dy=3*u*u*(p1[1]-p0[1])+6*u*t*(p2[1]-p1[1])+3*t*t*(p3[1]-p2[1])
    return x,y,math.atan2(dy,dx)
def companion2():
    g=[]
    for side in (-1,1):
        X=lambda x: 32+side*x
        p=[(X(4),57),(X(17),56),(X(25),46),(X(24.5),30)]; q=[(X(24.5),30),(X(24.2),24),(X(22.5),19.5),(X(19.5),16)]
        g.append(("line",f"M{p[0][0]} {p[0][1]} C{p[1][0]} {p[1][1]} {p[2][0]} {p[2][1]} {p[3][0]} {p[3][1]} C{q[1][0]} {q[1][1]} {q[2][0]} {q[2][1]} {q[3][0]} {q[3][1]}"))
        samples=[_bez(*p,t) for t in (0.3,0.55,0.8)]+[_bez(*q,t) for t in (0.15,0.55,0.95)]
        for k,(x,y,a) in enumerate(samples):
            for out in ((1,) if k==5 else (1,-1)):
                ang=a+out*side*math.radians(-40)*(-1)
                L,W=6.0 if k<5 else 5.0,2.2
                cx,cy=x+math.cos(ang)*L*0.5,y+math.sin(ang)*L*0.5
                tip=(x+math.cos(ang)*L,y+math.sin(ang)*L); nx,ny=-math.sin(ang)*W,math.cos(ang)*W
                g.append(("shape",f"M{x:.2f} {y:.2f} Q{cx+nx:.2f} {cy+ny:.2f} {tip[0]:.2f} {tip[1]:.2f} Q{cx-nx:.2f} {cy-ny:.2f} {x:.2f} {y:.2f} Z"))
    g.append(("shape","M20 54 C20 42.5 25.2 37.5 32 37.5 C38.8 37.5 44 42.5 44 54 Z"))
    g.append(("line","M27.5 46 L32 51 L36.5 46"))
    g.append(("shape",circ(32,27.5,7.6)))
    g.append(("shape",star(32,9.2,6.2,1.8,4)))
    return g
G["Companion"]=companion2()
G["Linkshell"]=linkpearl2()
