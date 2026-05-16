use serde::{Deserialize, Serialize};
use yewdux::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: usize,
    pub block_type: BlockType,
    pub content: String,
    pub prev: Option<usize>,
    pub next: Option<usize>,
}

impl Block {
    pub fn new(id: usize, block_type: BlockType) -> Self {
        Self {
            id,
            block_type,
            content: String::new(),
            prev: None,
            next: None,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block::new(0, BlockType::Paragraph)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum BlockType {
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Image,
    Citation,
    CodeBlock,
    BulletList,
    NumberedList,
    Quote,
    HorizontalRule,
    Introducao,
    Desenvolvimento,
    Conclusao,
    Teorema,
    Prova,
    Definicao,
    Exemplo,
    Observacao,
    CitacaoLonga,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Buffer {
    pub head: Option<usize>,
    pub tail: Option<usize>,
    pub blocks: std::collections::HashMap<usize, Block>,
    pub length: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            blocks: std::collections::HashMap::new(),
            length: 0,
        }
    }

    pub fn push_back(&mut self, block: Block) {
        let id = block.id;

        if let Some(tail_id) = self.tail {
            if let Some(b) = self.blocks.get_mut(&tail_id) {
                b.next = Some(id);
            }
        } else {
            self.head = Some(id);
        }

        self.blocks.insert(
            id,
            Block {
                prev: self.tail,
                ..block
            },
        );
        self.tail = Some(id);
        self.length += 1;
    }

    pub fn to_vec(&self) -> Vec<Block> {
        let mut result = Vec::new();
        let mut current = self.head;

        while let Some(id) = current {
            if let Some(block) = self.blocks.get(&id) {
                result.push(block.clone());
            }
            current = self.blocks.get(&id).and_then(|b| b.next);
        }

        result
    }

    pub fn insert_after(&mut self, block: Block, after_id: Option<usize>) -> usize {
        let id = block.id;

        if let Some(after) = after_id {
            if let Some(current) = self.blocks.get_mut(&after) {
                let next_id = current.next;
                current.next = Some(id);

                let mut new_block = block;
                new_block.prev = Some(after);
                new_block.next = next_id;

                if let Some(nid) = next_id {
                    if let Some(next) = self.blocks.get_mut(&nid) {
                        next.prev = Some(id);
                    }
                } else {
                    self.tail = Some(id);
                }

                self.blocks.insert(id, new_block);
            } else {
                self.push_back(block);
            }
        } else {
            if let Some(tail_id) = self.tail {
                if let Some(tail) = self.blocks.get_mut(&tail_id) {
                    tail.next = Some(id);
                }
                let mut new_block = block;
                new_block.prev = Some(tail_id);
                self.blocks.insert(id, new_block);
            } else {
                self.blocks.insert(id, Block { prev: None, ..block });
                self.head = Some(id);
            }
            self.tail = Some(id);
        }

        self.length += 1;
        id
    }

    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        let mut current = self.head;

        while let Some(id) = current {
            if let Some(block) = self.blocks.get(&id) {
                match block.block_type {
                    BlockType::Heading1 => {
                        markdown.push_str(&format!("# {}\n\n", block.content));
                    }
                    BlockType::Heading2 => {
                        markdown.push_str(&format!("## {}\n\n", block.content));
                    }
                    BlockType::Heading3 => {
                        markdown.push_str(&format!("### {}\n\n", block.content));
                    }
                    BlockType::Paragraph => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("{}\n\n", block.content));
                        }
                    }
                    BlockType::Image => {
                        markdown.push_str(&format!("![{}]({})\n\n", block.content, block.content));
                    }
                    BlockType::Citation => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("> {}\n\n", block.content));
                        }
                    }
                    BlockType::CodeBlock => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("```\n{}\n```\n\n", block.content));
                        }
                    }
                    BlockType::BulletList => {
                        markdown.push_str(&format!("- {}\n", block.content));
                    }
                    BlockType::NumberedList => {
                        markdown.push_str(&format!("1. {}\n", block.content));
                    }
                    BlockType::Quote => {
                        markdown.push_str(&format!("> {}\n\n", block.content));
                    }
                    BlockType::HorizontalRule => {
                        markdown.push_str("---\n\n");
                    }
                    BlockType::Introducao => {
                        markdown.push_str(&format!("# Introdução\n\n{}\n\n", block.content));
                    }
                    BlockType::Desenvolvimento => {
                        markdown.push_str(&format!("# Desenvolvimento\n\n{}\n\n", block.content));
                    }
                    BlockType::Conclusao => {
                        markdown.push_str(&format!("# Conclusão\n\n{}\n\n", block.content));
                    }
                    BlockType::Teorema => {
                        if !block.content.is_empty() {
                            markdown
                                .push_str(&format!("::: .theorem\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Prova => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .proof\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Definicao => {
                        if !block.content.is_empty() {
                            markdown.push_str(
                                &format!("::: .definition\n{}\n:::\n\n", block.content),
                            );
                        }
                    }
                    BlockType::Exemplo => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: .example\n{}\n:::\n\n", block.content));
                        }
                    }
                    BlockType::Observacao => {
                        if !block.content.is_empty() {
                            markdown.push_str(
                                &format!("::: .observation\n{}\n:::\n\n", block.content),
                            );
                        }
                    }
                    BlockType::CitacaoLonga => {
                        if !block.content.is_empty() {
                            markdown.push_str(&format!("::: citacao\n{}\n:::\n\n", block.content));
                        }
                    }
                }
            }
            current = self.blocks.get(&id).and_then(|b| b.next);
        }
        markdown
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub title: String,
    pub buffer: Buffer,
    pub file_path: Option<String>,
    pub is_dirty: bool,
    pub block_order: Vec<usize>,
    pub saved_content: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct SlashOption {
    pub block_type: Option<BlockType>,
    pub label: String,
    pub icon: &'static str,
    pub category: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct SlashCategory {
    pub name: String,
    pub options: Vec<SlashOption>,
}

#[derive(Clone, PartialEq, Store)]
pub struct EditorState {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub next_block_id: usize,
    pub show_slash_menu: bool,
    pub slash_menu_block_id: Option<usize>,
    pub focused_block_id: Option<usize>,
    pub show_save_modal: bool,
    pub save_modal_filename: String,
    pub xelatex_available: bool,
    pub xelatex_version: Option<String>,
    pub save_modal_export_type: ExportType,
    pub show_settings_modal: bool,
    pub dark_mode: bool,
    pub notification: Option<Notification>,
    pub show_citation_finder: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
    pub is_error: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorStateDto {
    pub tabs: Vec<Tab>,
    pub active_tab_id: usize,
    pub next_tab_id: usize,
    pub next_block_id: usize,
    pub show_settings_modal: bool,
    pub dark_mode: bool,
}

impl From<&EditorState> for EditorStateDto {
    fn from(state: &EditorState) -> Self {
        Self {
            tabs: state.tabs.clone(),
            active_tab_id: state.active_tab_id,
            next_tab_id: state.next_tab_id,
            next_block_id: state.next_block_id,
            show_settings_modal: false,
            dark_mode: false,
        }
    }
}

impl From<EditorStateDto> for EditorState {
    fn from(dto: EditorStateDto) -> Self {
        let mut state = Self::default();
        state.tabs = dto.tabs;
        state.active_tab_id = dto.active_tab_id;
        state.next_tab_id = dto.next_tab_id;
        state.next_block_id = dto.next_block_id;
        state.show_settings_modal = dto.show_settings_modal;
        state.dark_mode = dto.dark_mode;
        state
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportType {
    Markdown,
    Pdf,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut buffer = Buffer::new();
        buffer.push_back(Block::new(0, BlockType::Paragraph));

        Self {
            tabs: vec![Tab {
                id: 0,
                name: "Sem título.md".to_string(),
                title: "Sem título".to_string(),
                buffer,
                file_path: None,
                is_dirty: false,
                block_order: vec![0],
                saved_content: None,
            }],
            active_tab_id: 0,
            next_tab_id: 1,
            next_block_id: 1,
            show_slash_menu: false,
            slash_menu_block_id: None,
            focused_block_id: None,
            show_save_modal: false,
            save_modal_filename: "Sem título".to_string(),
            xelatex_available: false,
            xelatex_version: None,
            save_modal_export_type: ExportType::Markdown,
            show_settings_modal: false,
            dark_mode: false,
            notification: None,
            show_citation_finder: false,
        }
    }
}

pub fn get_slash_categories() -> Vec<SlashCategory> {
    vec![
        SlashCategory {
            name: "Básico".to_string(),
            options: vec![
                SlashOption {
                    block_type: Some(BlockType::Paragraph),
                    label: "Parágrafo".to_string(),
                    icon: "¶",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Heading1),
                    label: "Título 1".to_string(),
                    icon: "H1",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Heading2),
                    label: "Título 2".to_string(),
                    icon: "H2",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Heading3),
                    label: "Título 3".to_string(),
                    icon: "H3",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::BulletList),
                    label: "Lista com marcadores".to_string(),
                    icon: "•",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::NumberedList),
                    label: "Lista numerada".to_string(),
                    icon: "1.",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Quote),
                    label: "Citação".to_string(),
                    icon: "❝",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::CodeBlock),
                    label: "Código".to_string(),
                    icon: "</>",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Image),
                    label: "Imagem".to_string(),
                    icon: "🖼",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::HorizontalRule),
                    label: "Linha horizontal".to_string(),
                    icon: "—",
                    category: None,
                },
            ],
        },
        SlashCategory {
            name: "Estrutura do Texto".to_string(),
            options: vec![
                SlashOption {
                    block_type: Some(BlockType::Introducao),
                    label: "Introdução".to_string(),
                    icon: "#",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Desenvolvimento),
                    label: "Desenvolvimento".to_string(),
                    icon: "=",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Conclusao),
                    label: "Conclusão".to_string(),
                    icon: "✓",
                    category: None,
                },
            ],
        },
        SlashCategory {
            name: "Ambientes".to_string(),
            options: vec![
                SlashOption {
                    block_type: Some(BlockType::Teorema),
                    label: "Teorema".to_string(),
                    icon: "▢",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Prova),
                    label: "Prova".to_string(),
                    icon: "∎",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Definicao),
                    label: "Definição".to_string(),
                    icon: "≡",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Exemplo),
                    label: "Exemplo".to_string(),
                    icon: "ex",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::Observacao),
                    label: "Observação".to_string(),
                    icon: "i",
                    category: None,
                },
                SlashOption {
                    block_type: Some(BlockType::CitacaoLonga),
                    label: "Citação longa".to_string(),
                    icon: "❞",
                    category: None,
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test as test;

    fn make_block(id: usize, content: &str) -> Block {
        Block {
            id,
            block_type: BlockType::Paragraph,
            content: content.to_string(),
            prev: None,
            next: None,
        }
    }

    fn buffer_with_blocks(count: usize) -> Buffer {
        let mut buf = Buffer::new();
        for i in 0..count {
            buf.push_back(make_block(i, &format!("block {}", i)));
        }
        buf
    }

    #[test]
    fn test_insert_after_at_end_with_no_focus() {
        let mut buf = buffer_with_blocks(2);
        let new_block = make_block(5, "citation");
        let id = buf.insert_after(new_block, None);
        assert_eq!(id, 5);
        let vec = buf.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].content, "block 0");
        assert_eq!(vec[1].content, "block 1");
        assert_eq!(vec[2].content, "citation");
        assert_eq!(vec[2].prev, Some(1));
        assert_eq!(vec[1].next, Some(5));
    }

    #[test]
    fn test_insert_after_specific_block() {
        let mut buf = buffer_with_blocks(3);
        let new_block = make_block(10, "citation");
        let id = buf.insert_after(new_block, Some(0));
        assert_eq!(id, 10);
        let vec = buf.to_vec();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[0].content, "block 0");
        assert_eq!(vec[1].content, "citation");
        assert_eq!(vec[2].content, "block 1");
        assert_eq!(vec[3].content, "block 2");
        assert_eq!(vec[0].next, Some(10));
        assert_eq!(vec[1].prev, Some(0));
        assert_eq!(vec[1].next, Some(1));
        assert_eq!(vec[2].prev, Some(10));
    }

    #[test]
    fn test_insert_after_last_block() {
        let mut buf = buffer_with_blocks(3);
        let new_block = make_block(20, "citation");
        let id = buf.insert_after(new_block, Some(2));
        assert_eq!(id, 20);
        let vec = buf.to_vec();
        assert_eq!(vec.len(), 4);
        assert_eq!(vec[3].content, "citation");
        assert_eq!(vec[2].next, Some(20));
        assert_eq!(vec[3].prev, Some(2));
        assert_eq!(vec[3].next, None);
        assert_eq!(buf.tail, Some(20));
    }

    #[test]
    fn test_insert_after_empty_buffer() {
        let mut buf = Buffer::new();
        let new_block = make_block(0, "citation");
        let id = buf.insert_after(new_block, None);
        assert_eq!(id, 0);
        let vec = buf.to_vec();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0].content, "citation");
        assert_eq!(buf.head, Some(0));
        assert_eq!(buf.tail, Some(0));
    }

    #[test]
    fn test_insert_after_maintains_linked_list_integrity() {
        let mut buf = buffer_with_blocks(5);
        let new_block = make_block(99, "new");
        buf.insert_after(new_block, Some(2));

        let mut ids = Vec::new();
        let mut cur = buf.head;
        while let Some(id) = cur {
            ids.push(id);
            cur = buf.blocks.get(&id).and_then(|b| b.next);
        }
        assert_eq!(ids, vec![0, 1, 2, 99, 3, 4]);

        let mut prev_ids = Vec::new();
        let mut cur = buf.tail;
        while let Some(id) = cur {
            prev_ids.push(id);
            cur = buf.blocks.get(&id).and_then(|b| b.prev);
        }
        prev_ids.reverse();
        assert_eq!(prev_ids, vec![0, 1, 2, 99, 3, 4]);
    }

    #[test]
    fn test_insert_after_focused_block_flow() {
        let mut buf = buffer_with_blocks(2);
        let focused_id = Some(0usize);

        let insert_after = focused_id.and_then(|fid| {
            None // simulate no match
        });

        let new_block = make_block(10, "citation");
        buf.insert_after(new_block, insert_after);

        let vec = buf.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[2].content, "citation");
    }
}
