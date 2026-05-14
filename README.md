# Hazel

A clean, distraction-free block editor for creating ABNT-compliant academic documents. Write in blocks like Notion, export to `.md` or `.pdf` with proper Brazilian academic formatting (abnTex2).

<img width="1672" height="949" alt="image" src="https://github.com/user-attachments/assets/42a254f3-037a-4947-80e2-00af125c9d11" />

## Features

- **Block-based editing** — Write in blocks, not a single text field. Type `/` to access slash commands and change block types.
- **Slash commands** — Quick access to all block types:
  - Basic: Paragraph, Headings (H1-H3), Lists, Quotes, Code, Images
  - ABNT Structure: Introdução, Desenvolvimento, Conclusão
  - Environments: Teorema, Prova, Definição, Exemplo, Observação, Citação longa
- **Multiple tabs** — Work on several documents at once
- **Auto-save** — Your work is saved automatically to local storage
- **Dark mode** — Toggle via `Esc` key
- **Export options**:
  - Markdown (`.md`)
  - PDF with ABNT formatting via abntex2

## Installation

### Prerequisites

Install these system dependencies before running Hazel:

2. **xelatex** — Part of [TeX Live](https://www.tug.org/texlive/) or [MiKTeX](https://miktex.org/)

### Build from source

```bash
# Clone the repository
git clone https://github.com/eduardolsoares/hazel.git
cd hazel

# Build the Tauri backend
cargo build --manifest-path src-tauri/Cargo.toml

# Build the frontend and serve
trunk serve
```

Or use the provided build workflow:

```bash
# Full production build
cargo tauri build
```

See [releases](https://github.com/eduardolsoares/hazel/releases/) tab for wizard installers.

## Usage

### Getting started

1. Launch the application
2. Click on the title to rename your document
3. Start typing in the first block
4. Press `Enter` to create a new block
5. Type `/` to open the slash command menu

### Slash commands

Type `/` in any block to access:

| Command | Description |
|---------|-------------|
| `/` | Open command menu |
| Parágrafo | Standard text paragraph |
| Título 1/2/3 | Section headings |
| Lista com marcadores | Bullet point list |
| Lista numerada | Numbered list |
| Citação | Block quote |
| Código | Code block |
| Imagem | Image placeholder |
| Linha horizontal | Horizontal rule |
| Introdução | ABNT introduction section |
| Desenvolvimento | ABNT development section |
| Conclusão | ABNT conclusion section |
| Teorema | Theorem environment |
| Prova | Proof environment |
| Definição | Definition environment |
| Exemplo | Example environment |
| Observação | Observation environment |
| Citação longa | Long quotation (ABNT) |

### Keyboard shortcuts

| Shortcut | Action |
|---------|--------|
| `Ctrl+S` | Save / Export dialog |
| `Esc` | Toggle dark mode |
| `Enter` | New block |
| `Backspace` | Delete empty block |
| `↑` / `↓` | Navigate blocks |

### Exporting

Press `Ctrl+S` or click the save button to open the export dialog:

1. **Export as Markdown** — Saves as `.md` file
3. **Export as PDF** — Generates PDF with ABNT formatting (requires xelatex)

The PDF uses the abntex2 template with:
- Font: Times New Roman, 12pt
- Line spacing: 1.5
- Margins: 2cm (top/right/bottom), 3cm (left)
- Justified text
- Paragraph indent: 1.25cm


## License

See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or pull request on GitHub.

## Acknowledgments

- [abnTeX2](https://github.com/abntex/abntex2) — ABNT LaTeX templates
- [Pandoc](https://pandoc.org/) — Document converter
- [Yew](https://yew.rs/) — Rust WebAssembly frontend
- [Tauri](https://tauri.app/) — Desktop app framework
