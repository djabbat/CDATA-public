#!/usr/bin/env python3
"""Convert EIC_Pathfinder_CDATA_PartB.md to PDF via WeasyPrint."""

import markdown
from weasyprint import HTML, CSS
import os

WORKDIR = "/home/oem/Desktop/Horizon"
MD_FILE  = os.path.join(WORKDIR, "EIC_Pathfinder_CDATA_PartB.md")
PDF_FILE = os.path.join(WORKDIR, "EIC_Pathfinder_CDATA_PartB.pdf")

with open(MD_FILE, encoding="utf-8") as f:
    md_text = f.read()

html_body = markdown.markdown(md_text, extensions=["tables", "fenced_code"])

html = f"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  body {{
    font-family: 'Calibri', 'Arial', sans-serif;
    font-size: 10.5pt;
    line-height: 1.5;
    color: #1a1a1a;
    margin: 0;
    padding: 0;
  }}
  h1 {{
    font-size: 14pt;
    color: #00509e;
    border-bottom: 2px solid #c9a84c;
    padding-bottom: 4px;
    margin-top: 24px;
    margin-bottom: 6px;
  }}
  h2 {{
    font-size: 12pt;
    color: #0f1f3d;
    margin-top: 18px;
    margin-bottom: 4px;
  }}
  h3 {{
    font-size: 11pt;
    color: #0f1f3d;
    margin-top: 12px;
    margin-bottom: 3px;
  }}
  h4 {{
    font-size: 10.5pt;
    color: #444;
    font-style: italic;
    margin-top: 8px;
    margin-bottom: 2px;
  }}
  p {{ margin: 4px 0 6px 0; }}
  em {{ color: #555; font-style: italic; }}
  strong {{ color: #0f1f3d; }}
  table {{
    border-collapse: collapse;
    width: 100%;
    margin: 10px 0 14px 0;
    font-size: 9.5pt;
  }}
  th {{
    background: #0f1f3d;
    color: #fff;
    padding: 5px 8px;
    text-align: left;
    font-size: 9pt;
  }}
  td {{
    border: 1px solid #ccc;
    padding: 4px 8px;
    vertical-align: top;
  }}
  tr:nth-child(even) td {{ background: #f4f7fb; }}
  ul, ol {{ margin: 4px 0; padding-left: 18px; }}
  li {{ margin: 2px 0; }}
  blockquote {{
    border-left: 3px solid #c9a84c;
    margin: 8px 0;
    padding: 4px 10px;
    color: #444;
    font-style: italic;
    background: #fffef5;
  }}
  code {{
    font-family: 'Courier New', monospace;
    font-size: 8pt;
    background: #f5f5f5;
    padding: 1px 3px;
  }}
  pre {{
    font-family: 'Courier New', monospace;
    font-size: 7.5pt;
    background: #f5f5f5;
    padding: 8px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
  }}
  hr {{
    border: none;
    border-top: 1px solid #ddd;
    margin: 14px 0;
  }}
  /* Title box */
  .title-box {{
    background: #0f1f3d;
    color: white;
    padding: 16px;
    text-align: center;
    margin-bottom: 12px;
    border-radius: 3px;
  }}
</style>
</head>
<body>
<div class="title-box">
  <strong style="font-size:14pt;color:white;">Proposal Part B: Technical Description</strong><br>
  <span style="font-size:11pt;color:#c9a84c;">CDATA — Centriolar Damage Accumulation Theory of Ageing</span><br>
  <span style="font-size:9pt;color:#aac4e0;">EIC Pathfinder Open 2026 | Phasis Academy + GTU | PI: Jaba Tkemaladze MD | €2.5M / 36 months</span>
</div>
{html_body}
</body>
</html>"""

css = CSS(string="""
  @page {
    size: A4;
    margin: 2cm 2.2cm 2cm 2.2cm;
    @bottom-center {
      content: "CDATA EIC Pathfinder Open 2026 | Phasis Academy + GTU — Confidential";
      font-size: 7.5pt;
      color: #999;
    }
    @top-right {
      content: counter(page);
      font-size: 8pt;
      color: #999;
    }
  }
""")

HTML(string=html, base_url="/").write_pdf(PDF_FILE, stylesheets=[css])
print(f"✅ PDF: {PDF_FILE}")
size_kb = os.path.getsize(PDF_FILE) // 1024
print(f"   Size: {size_kb} KB")
