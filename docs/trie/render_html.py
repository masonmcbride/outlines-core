"""Render the notebook's explanation without notebook UI scaffolding."""
import html
import re
import markdown


def prose(text):
    equations=[]
    def protect(match):
        equations.append(html.escape(match.group()))
        return f'MATHPLACEHOLDER{len(equations)-1}END'
    text=re.sub(r'\$\$[\s\S]+?\$\$|\$(?!\$)[^\n$]+\$',protect,text)
    body=markdown.markdown(text,extensions=['fenced_code','tables'])
    for i,equation in enumerate(equations):
        body=body.replace(f'MATHPLACEHOLDER{i}END',equation)
    return body


def export_html(notebook,path):
    parts=[]
    for cell in notebook.cells:
        if cell.cell_type=='markdown':
            parts.append(prose(cell.source))
        elif cell.cell_type=='code':
            for output in cell.outputs:
                data=output.get('data',{})
                if 'image/svg+xml' in data:
                    parts.append('<div class="figure">'+data['image/svg+xml']+'</div>')
                elif 'image/png' in data:
                    parts.append('<img src="figures/preparation.png" alt="Qwen index-construction time with preparation included or reused">')
    head="""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Read each token prefix once — Outlines Core</title>
<style>
body{max-width:900px;margin:40px auto;padding:0 24px;font:17px/1.65 system-ui;color:#26333d;background:white}
h1,h2{line-height:1.25}h1{font-size:34px}h2{font-size:23px;margin-top:32px}
a{color:#246b8f}pre{padding:16px;background:#f3f5f7;overflow:auto;font:13px/1.5 ui-monospace,monospace}
code{font-family:ui-monospace,monospace}img,.figure svg{max-width:100%;height:auto}
details{padding:12px 0;border-top:1px solid #ddd}summary{cursor:pointer;font-weight:600}
table{border-collapse:collapse}td,th{padding:8px;border-bottom:1px solid #ddd}
@media(max-width:600px){body{padding:0 16px;font-size:16px}h1{font-size:28px}}
</style>
<script type="text/x-mathjax-config">
MathJax.Hub.Config({tex2jax:{inlineMath:[['$','$']],displayMath:[['$$','$$']],processEscapes:true}});
</script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.7/MathJax.js?config=TeX-AMS_CHTML"></script>
</head>
<body>
<main>
"""
    path.write_text(head+'\n'.join(parts)+'\n</main>\n</body>\n</html>\n')
