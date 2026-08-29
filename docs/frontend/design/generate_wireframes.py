from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import A4, landscape
from reportlab.lib.colors import Color
import os

# Create directory if it doesn't exist
os.makedirs("d:/Star/docs/frontend/design", exist_ok=True)

# Design system colors
bg_dark = Color(10/255, 13/255, 18/255)
bg_soft = Color(22/255, 27/255, 34/255)
bg_card = Color(28/255, 33/255, 40/255)
border_line = Color(33/255, 38/255, 45/255)
ink = Color(230/255, 237/255, 243/255)
ink_dim = Color(125/255, 133/255, 144/255)
accent = Color(47/255, 129/255, 247/255)
ok = Color(63/255, 185/255, 80/255)
warn = Color(210/255, 153/255, 34/255)
err = Color(248/255, 81/255, 73/255)
info = Color(88/255, 166/255, 255/255)

# Wireframe colors
wf_bg = Color(0.95, 0.95, 0.97)
wf_accent = Color(0.18, 0.50, 0.97)
wf_border = Color(0.78, 0.80, 0.84)
wf_text = Color(0.1, 0.1, 0.1)

def draw_rulers(c):
    c.setStrokeColor(wf_border)
    c.setLineWidth(0.5)
    c.setFont("Helvetica", 6)
    c.setFillColor(wf_text)
    # x axis
    for x in range(0, 842, 50):
        c.line(x, 595, x, 590)
        c.drawString(x + 2, 585, str(x))
    # y axis
    for y in range(0, 595, 50):
        c.line(0, y, 5, y)
        c.drawString(7, y + 2, str(y))

def draw_region(c, name, x, y, w, h, fill=wf_bg, stroke=wf_border, font_color=wf_text):
    c.setFillColor(fill)
    c.setStrokeColor(stroke)
    c.setLineWidth(1)
    c.rect(x, y, w, h, fill=1, stroke=1)
    
    # Text
    c.setFillColor(font_color)
    c.setFont("Helvetica-Bold", 8)
    c.drawCentredString(x + w/2, y + h/2 + 2, name)
    c.setFont("Helvetica", 6)
    c.setFillColor(Color(0.4, 0.4, 0.4))
    c.drawCentredString(x + w/2, y + h/2 - 6, f"({x},{y}) {w}x{h}")

def draw_tabs(c, x, y, w, h, tabs, active_index=0):
    draw_region(c, "Tabs Container", x, y, w, h)
    tab_w = w / len(tabs)
    for i, tab in enumerate(tabs):
        tx = x + i * tab_w
        if i == active_index:
            draw_region(c, tab, tx, y, tab_w, h, fill=wf_accent, font_color=Color(1,1,1))
        else:
            draw_region(c, tab, tx, y, tab_w, h)

# Setup
file_path = "d:/Star/docs/frontend/design/ui-wireframes.pdf"
c = canvas.Canvas(file_path, pagesize=landscape(A4))

# Page 1: Title Page
c.setFont("Helvetica-Bold", 24)
c.setFillColor(wf_text)
c.drawString(100, 450, "Star Platform — UI Wireframes v1.0")
c.setFont("Helvetica", 14)
c.drawString(100, 420, "带坐标标注的 UI 原型图 / Annotated UI Prototype")
c.setFont("Helvetica", 12)
c.drawString(100, 390, "Date: 2026-08-29")
c.drawString(100, 370, "Author: 架构师 (Mavis 接手 agent per DEC-008)")

c.drawString(100, 330, "Contents:")
contents = [
    "Page 2: Coordinate System Legend",
    "Page 3: /work-item page",
    "Page 4: /agent page",
    "Page 5: /notification page",
    "Page 6: /project page",
    "Page 7: /feedback page",
    "Page 8: Component Reference Sheet",
    "Page 9: /analytics (Charts) page",
    "Page 10: /planning page"
]
for i, line in enumerate(contents):
    c.drawString(120, 300 - i*20, line)
draw_rulers(c)
c.showPage()

# Page 2: Legend
draw_rulers(c)
c.setFont("Helvetica-Bold", 16)
c.setFillColor(wf_text)
c.drawString(50, 500, "Coordinate System Legend")
c.setFont("Helvetica", 10)
c.drawString(50, 480, "Grid size: 842x595 points (A4 Landscape)")
c.drawString(50, 465, "Origin (0,0) is at the Bottom-Left corner.")
c.drawString(50, 450, "Tick marks every 50pt.")

draw_region(c, "Example Component", 50, 300, 200, 100)
c.setFont("Helvetica", 10)
c.drawString(270, 350, "<- Example bounding box with name and coordinate annotation.")

c.drawString(50, 250, "Color Legend:")
draw_region(c, "Background", 50, 210, 80, 30, fill=wf_bg)
draw_region(c, "Accent/Active", 140, 210, 80, 30, fill=wf_accent, font_color=Color(1,1,1))
draw_region(c, "Border", 230, 210, 80, 30, fill=wf_border)

c.setFont("Helvetica", 10)
c.setFillColor(wf_text)
c.drawString(50, 170, "Typography Scale:")
c.setFont("Helvetica-Bold", 14)
c.drawString(50, 150, "Heading (14pt)")
c.setFont("Helvetica", 10)
c.drawString(50, 130, "Body (10pt)")
c.setFont("Helvetica", 8)
c.drawString(50, 115, "Small / Label (8pt)")
c.setFont("Helvetica", 6)
c.drawString(50, 105, "Annotation (6pt)")
c.showPage()

# Page 3: /work-item
draw_rulers(c)
draw_region(c, "Sidebar | 220px", 0, 0, 220, 531)
draw_region(c, "TopBar | AppHeader", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Tab 1", "Tab 2", "Tab 3"])
draw_region(c, "Pill filters row", 228, 423, 606, 28)
draw_region(c, "Table", 228, 0, 408, 415)
draw_region(c, "Detail drawer", 644, 0, 190, 415)
c.showPage()

# Page 4: /agent
draw_rulers(c)
draw_region(c, "Sidebar", 0, 0, 220, 531)
draw_region(c, "TopBar", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Tab 1", "Tab 2", "Tab 3", "Tab 4"])
draw_region(c, "Stat cards row", 228, 403, 606, 40)
draw_region(c, "Agent table", 228, 100, 606, 295)
draw_region(c, "Budget burn bars", 228, 0, 606, 92)
c.showPage()

# Page 5: /notification
draw_rulers(c)
draw_region(c, "Sidebar", 0, 0, 220, 531)
draw_region(c, "TopBar", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Tab 1", "Tab 2", "Tab 3", "Tab 4"])
draw_region(c, "Stat cards", 228, 411, 606, 32)
draw_region(c, "Filter/actions row", 228, 387, 606, 16)
draw_region(c, "Notification table", 228, 50, 400, 329)
draw_region(c, "Detail drawer", 636, 50, 198, 329)
c.showPage()

# Page 6: /project
draw_rulers(c)
draw_region(c, "Sidebar", 0, 0, 220, 531)
draw_region(c, "TopBar", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Tab 1", "Tab 2", "Tab 3"])
draw_region(c, "Card 1", 228, 220, 192, 223)
draw_region(c, "Card 2", 428, 220, 192, 223)
draw_region(c, "Card 3", 628, 220, 192, 223)
c.showPage()

# Page 7: /feedback
draw_rulers(c)
draw_region(c, "Sidebar", 0, 0, 220, 531)
draw_region(c, "TopBar", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Tab 1", "Tab 2", "Tab 3"])
draw_region(c, "Stat row", 228, 411, 606, 32)
draw_region(c, "Feedback list", 228, 50, 398, 353)
draw_region(c, "Detail panel", 634, 50, 200, 353)
c.showPage()

# Page 8: Component Reference Sheet
draw_rulers(c)
c.setFont("Helvetica-Bold", 16)
c.setFillColor(wf_text)
c.drawString(50, 550, "Component Reference Sheet")

# Tabs
c.drawString(50, 520, "Tabs (underline variant)")
draw_tabs(c, 50, 480, 200, 30, ["Tab 1", "Tab 2", "Tab 3"])

# StatusPill
c.drawString(300, 520, "StatusPill variants")
draw_region(c, "todo", 300, 480, 40, 20, fill=wf_bg)
draw_region(c, "in_progress", 350, 480, 60, 20, fill=accent, font_color=Color(1,1,1))
draw_region(c, "done", 420, 480, 40, 20, fill=ok, font_color=Color(1,1,1))
draw_region(c, "warn", 470, 480, 40, 20, fill=warn, font_color=Color(1,1,1))
draw_region(c, "err", 520, 480, 40, 20, fill=err, font_color=Color(1,1,1))
draw_region(c, "info", 570, 480, 40, 20, fill=info, font_color=Color(1,1,1))

# Stat card
c.drawString(50, 430, "Stat Card")
draw_region(c, "Label", 50, 370, 150, 50)
draw_region(c, "Value", 70, 380, 110, 30, fill=Color(1,1,1))

# Kanban card
c.drawString(300, 430, "Kanban Card")
draw_region(c, "Kanban Card Bg", 300, 320, 200, 100)
draw_region(c, "Title", 310, 390, 180, 20, fill=Color(1,1,1))
draw_region(c, "Pill", 310, 360, 40, 15, fill=info)
draw_region(c, "Pri", 360, 360, 30, 15, fill=warn)
draw_region(c, "SP", 460, 360, 20, 15, fill=Color(1,1,1))

# Detail drawer
c.drawString(50, 310, "Detail Drawer")
draw_region(c, "Drawer Base", 50, 100, 200, 200)
draw_region(c, "Header", 50, 270, 200, 30, fill=Color(0.9, 0.9, 0.9))
draw_region(c, "dl rows", 60, 150, 180, 110, fill=Color(1,1,1))
draw_region(c, "Action btn", 60, 110, 180, 30, fill=wf_accent, font_color=Color(1,1,1))

# PageHeader
c.drawString(300, 270, "PageHeader")
draw_region(c, "PageHeader Base", 300, 210, 400, 50)
draw_region(c, "Icon", 310, 220, 30, 30, fill=Color(1,1,1))
draw_region(c, "Title", 350, 235, 100, 20, fill=Color(1,1,1))
draw_region(c, "Subtitle", 350, 215, 150, 15, fill=Color(1,1,1))
draw_region(c, "Badge", 460, 235, 30, 20, fill=info)

c.showPage()

# Page 9: /analytics (Charts) page
draw_rulers(c)
draw_region(c, "Sidebar", 0, 0, 220, 531)
draw_region(c, "TopBar", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Burndown", "Gantt", "Velocity", "Cost Trend", "Leaderboard"], active_index=1)
draw_region(c, "KPI stat cards row", 228, 411, 606, 32)
draw_region(c, "Y-axis labels", 228, 0, 100, 403)
draw_region(c, "Timeline header", 328, 385, 506, 18)
draw_region(c, "Bar 1 (Sprint 1)", 330, 340, 200, 18)
draw_region(c, "Bar 2 (Sprint 2)", 430, 310, 250, 18)
draw_region(c, "Milestone diamond", 490, 270, 8, 18)
draw_region(c, "Work item bar 1", 342, 230, 120, 14)
draw_region(c, "Work item bar 2", 382, 200, 180, 14)
c.showPage()

# Page 10: /planning page
draw_rulers(c)
draw_region(c, "Sidebar", 0, 0, 220, 531)
draw_region(c, "TopBar", 0, 531, 842, 64)
draw_region(c, "PageHeader", 220, 487, 622, 44)
draw_tabs(c, 220, 451, 622, 36, ["Sprints", "Calendar", "Milestones"], active_index=0)
draw_region(c, "Sprint card 1", 228, 350, 606, 85)
draw_region(c, "Sprint card 2", 228, 255, 606, 85)
draw_region(c, "Sprint card 3", 228, 160, 606, 85)
draw_region(c, "New Sprint button", 228, 120, 120, 32)
c.showPage()

c.save()
