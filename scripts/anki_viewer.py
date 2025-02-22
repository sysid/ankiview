import json
import webbrowser
import tempfile
import os
import urllib.request
import urllib.error
import re

class AnkiConnect:
    def __init__(self, host="localhost", port=8765):
        self.url = f"http://{host}:{port}"

    def request(self, action, **params):
        payload = json.dumps({
            "action": action,
            "version": 6,
            "params": params
        }).encode('utf-8')

        try:
            request = urllib.request.Request(self.url, payload)
            response = urllib.request.urlopen(request)
            return json.loads(response.read().decode('utf-8'))
        except urllib.error.URLError as e:
            print(f"Error connecting to Anki: {e}")
            print("Make sure Anki is running and AnkiConnect addon is installed.")
            return None

def get_note(note_id):
    """Retrieve a note from Anki using AnkiConnect."""
    anki = AnkiConnect()
    response = anki.request("notesInfo", notes=[note_id])
    if response and not response.get("error"):
        return response
    return None

def extract_latex(content):
    """Extract LaTeX from code blocks and return only the LaTeX content."""
    pattern = r'```(?:tex|latex)?\n(\$\$[\s\S]*?\$\$)\n```'
    return re.sub(pattern, r'\1', content)

def create_card_html(note_data):
    """Create HTML content for the note."""
    note = note_data['result'][0]

    # Extract LaTeX from code blocks
    front_content = extract_latex(note['fields']['Front']['value'])
    back_content = extract_latex(note['fields']['Back']['value'])

    # Format tags
    tags_html = ' '.join(f'<span class="tag">{tag}</span>' for tag in note.get('tags', []))

    html_template = """<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Anki Note Viewer</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.7/MathJax.js?config=TeX-AMS_HTML"></script>
    <script type="text/x-mathjax-config">
        MathJax.Hub.Config({{
            tex2jax: {{
                inlineMath: [['$', '$']],
                displayMath: [['$$', '$$']],
                processEscapes: true
            }}
        }});
    </script>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 2rem auto;
            padding: 0 1rem;
            background-color: #f5f5f5;
        }}
        .card {{
            background: white;
            border-radius: 8px;
            padding: 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        code {{
            background-color: #f0f0f0;
            padding: 2px 4px;
            border-radius: 3px;
            font-family: monospace;
        }}
        .card-front {{
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 2px solid #eee;
        }}
        .note-info {{
            margin-top: 1rem;
            padding-top: 1rem;
            border-top: 1px solid #eee;
            font-size: 0.9em;
            color: #666;
        }}
        .tag {{
            display: inline-block;
            background: #e9ecef;
            padding: 2px 8px;
            border-radius: 4px;
            margin-right: 4px;
            font-size: 0.8em;
        }}
    </style>
</head>
<body>
    <div class="card">
        <div class="card-front">
            <h2>Question</h2>
            {front}
        </div>
        <div class="card-back">
            <h2>Answer</h2>
            {back}
        </div>
        <div class="note-info">
            <div>Note ID: {note_id}</div>
            <div>Model: {model}</div>
            <div>Tags: {tags}</div>
        </div>
    </div>
    <script>
        MathJax.Hub.Queue(["Typeset", MathJax.Hub]);
    </script>
</body>
</html>
"""

    return html_template.format(
        front=front_content,
        back=back_content,
        note_id=note['noteId'],
        model=note.get('modelName', 'Unknown'),
        tags=tags_html or '<span class="tag">No tags</span>'
    )

def view_note(note_id):
    """Retrieve and display an Anki note in the default web browser."""
    note_data = get_note(note_id)
    if not note_data:
        print("Failed to retrieve note data")
        return None

    html_content = create_card_html(note_data)

    with tempfile.NamedTemporaryFile('w', delete=False, suffix='.html', encoding='utf-8') as f:
        f.write(html_content)
        temp_path = f.name

    webbrowser.open('file://' + os.path.abspath(temp_path))
    return temp_path

def main():
    import argparse
    parser = argparse.ArgumentParser(description='View Anki notes in your web browser')
    parser.add_argument('note_id', type=int, help='The Anki note ID to view')
    args = parser.parse_args()

    temp_file = view_note(args.note_id)
    if temp_file:
        print(f"Opened note in browser. Temporary file created at: {temp_file}")

if __name__ == "__main__":
    # python z.py 1727417322608
    main()
