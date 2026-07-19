import re

with open('src/app.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Make all fields in CadsdState public
match = re.search(r'(pub struct CadsdState \{)(.*?)(\})', content, re.DOTALL)
if match:
    body = match.group(2)
    # Replace any leading space + field name with 'pub ' + field name
    def repl(m):
        return m.group(1) + 'pub ' + m.group(2)
    new_body = re.sub(r'(?m)^(\s+)(?!pub )([a-z_]+:)', repl, body)
    content = content[:match.start()] + match.group(1) + new_body + match.group(3) + content[match.end():]

# Replace show_settings_panel
content = re.sub(r'(?s)fn show_settings_panel\(ui: &mut egui::Ui, state: &mut CadsdState\) \{.*?^}', 'fn show_settings_panel(ui: &mut egui::Ui, state: &mut CadsdState) {\n    crate::ui::show_settings_panel(ui, state);\n}', content, flags=re.MULTILINE)

with open('src/app.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed app.rs')
