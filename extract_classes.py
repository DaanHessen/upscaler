import os, re
with open('class_lines.txt', 'w', encoding='utf-8') as out:
    for root, dirs, files in os.walk('rust-frontend/src'):
        for file in files:
            if file.endswith('.rs'):
                with open(os.path.join(root, file), 'r', encoding='utf-8') as f:
                    content = f.read()
                    if 'class' in content:
                        out.write('--- ' + file + '\n')
                        for line in content.split('\n'):
                            if 'class' in line:
                                out.write(line.strip() + '\n')
