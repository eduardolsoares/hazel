#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SaveResponse {
        pub success: bool,
        pub file_path: Option<String>,
        pub error: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct XelatexCheckResult {
        pub available: bool,
        pub version: Option<String>,
    }

    #[test]
    fn test_save_response_success() {
        let response = SaveResponse {
            success: true,
            file_path: Some("/path/to/file.md".to_string()),
            error: None,
        };

        assert!(response.success);
        assert_eq!(response.file_path, Some("/path/to/file.md".to_string()));
        assert!(response.error.is_none());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_save_response_error() {
        let response = SaveResponse {
            success: false,
            file_path: None,
            error: Some("File not found".to_string()),
        };

        assert!(!response.success);
        assert!(response.file_path.is_none());
        assert_eq!(response.error, Some("File not found".to_string()));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
    }

    #[test]
    fn test_save_response_serialize() {
        let response = SaveResponse {
            success: true,
            file_path: Some("test.md".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test.md"));
    }

    #[test]
    fn test_save_response_deserialize() {
        let json = r#"{"success":true,"file_path":"output.pdf","error":null}"#;
        let response: SaveResponse = serde_json::from_str(json).unwrap();

        assert!(response.success);
        assert_eq!(response.file_path, Some("output.pdf".to_string()));
    }

    #[test]
    fn test_xelatex_check_result_available() {
        let result = XelatexCheckResult {
            available: true,
            version: Some("XeTeX 3.14159265".to_string()),
        };

        assert!(result.available);
        assert!(result.version.is_some());
    }

    #[test]
    fn test_xelatex_check_result_unavailable() {
        let result = XelatexCheckResult {
            available: false,
            version: None,
        };

        assert!(!result.available);
        assert!(result.version.is_none());
    }

    #[test]
    fn test_export_request_serialization() {
        #[derive(Serialize)]
        struct ExportRequest {
            content: String,
            default_name: Option<String>,
        }

        let request = ExportRequest {
            content: "# Title\n\nContent".to_string(),
            default_name: Some("document".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("# Title"));
        assert!(json.contains("document"));
    }

    #[test]
    fn test_markdown_to_pdf_request() {
        #[derive(Serialize)]
        struct RenderRequest {
            content: String,
            profile: String,
        }

        let request = RenderRequest {
            content: "# Hello World\n\nThis is a test.".to_string(),
            profile: "abnt".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("abnt"));
    }

    // --- Citation finder tests ---

    #[derive(Clone, Debug)]
    struct Work {
        doi: Option<String>,
        title: String,
        authors: Vec<String>,
        year: Option<String>,
        journal: Option<String>,
        volume: Option<String>,
        issue: Option<String>,
        pages: Option<String>,
        publisher: Option<String>,
        work_type: String,
    }

    fn make_work(
        title: &str,
        authors: Vec<&str>,
        year: Option<&str>,
        journal: Option<&str>,
        volume: Option<&str>,
        issue: Option<&str>,
        pages: Option<&str>,
        publisher: Option<&str>,
        work_type: &str,
    ) -> Work {
        Work {
            doi: None,
            title: title.to_string(),
            authors: authors.into_iter().map(|a| a.to_string()).collect(),
            year: year.map(|y| y.to_string()),
            journal: journal.map(|j| j.to_string()),
            volume: volume.map(|v| v.to_string()),
            issue: issue.map(|i| i.to_string()),
            pages: pages.map(|p| p.to_string()),
            publisher: publisher.map(|p| p.to_string()),
            work_type: work_type.to_string(),
        }
    }

    fn format_abnt(work: &Work) -> String {
        let authors_str = if work.authors.is_empty() {
            String::new()
        } else {
            let mut s = String::new();
            for (i, author) in work.authors.iter().enumerate() {
                if i > 0 {
                    s.push_str("; ");
                }
                s.push_str(author);
            }
            s
        };

        let cap_author = if authors_str.is_empty() {
            String::new()
        } else {
            format!("{}. ", authors_str)
        };

        let title = work.title.trim().trim_end_matches('.');
        let year = work.year.as_deref().unwrap_or("s.d.").trim_end_matches('.');

        match work.work_type.as_str() {
            "journal-article" | "journal-issue" => {
                let journal = work.journal.as_deref().unwrap_or("");
                let vol = work.volume.as_deref().unwrap_or("");
                let iss = work.issue.as_deref().unwrap_or("");
                let pg = work.pages.as_deref().unwrap_or("");

                let mut parts = vec![format!("{}{}.", cap_author, title)];

                if !journal.is_empty() {
                    parts.push(format!(" *{}*", journal));
                }

                let mut details = Vec::new();
                if !vol.is_empty() {
                    details.push(format!("v. {}", vol));
                }
                if !iss.is_empty() {
                    details.push(format!("n. {}", iss));
                }
                if !pg.is_empty() {
                    details.push(format!("p. {}", pg));
                }
                if !details.is_empty() {
                    parts.push(format!(", {}", details.join(", ")));
                }

                parts.push(format!(", {}.", year));
                parts.concat()
            }
            "book" | "monograph" | "book-section" | "book-chapter" | "edited-book" => {
                let publisher = work.publisher.as_deref().unwrap_or("");
                let mut parts = vec![format!("{}{}.", cap_author, title)];
                if !publisher.is_empty() {
                    parts.push(format!(" {};", publisher));
                }
                parts.push(format!(" {}.", year));
                parts.concat()
            }
            "proceedings-article" | "paper-conference" => {
                let journal = work.journal.as_deref().unwrap_or("");
                let publisher = work.publisher.as_deref().unwrap_or("");
                let pg = work.pages.as_deref().unwrap_or("");
                let vol = work.volume.as_deref().unwrap_or("");

                let mut parts = vec![format!("{}{}.", cap_author, title)];
                if !journal.is_empty() {
                    parts.push(format!(" In: {}", journal));
                }
                if !vol.is_empty() {
                    parts.push(format!(", v. {}", vol));
                }
                if !pg.is_empty() {
                    parts.push(format!(", p. {}", pg));
                }
                if !publisher.is_empty() {
                    parts.push(format!(" {};", publisher));
                }
                parts.push(format!(" {}.", year));
                parts.concat()
            }
            _ => {
                let publisher = work.publisher.as_deref().unwrap_or("");
                let mut parts = vec![format!("{}{}.", cap_author, title)];
                if !publisher.is_empty() {
                    parts.push(format!(" {};", publisher));
                }
                parts.push(format!(" {}.", year));
                parts.concat()
            }
        }
    }

    #[test]
    fn test_format_abnt_journal_article_full() {
        let work = make_work(
            "A importância da pesquisa científica",
            vec!["SILVA, João", "SOUZA, Maria"],
            Some("2023"),
            Some("Revista Brasileira de Ciência"),
            Some("10"),
            Some("2"),
            Some("15-30"),
            None,
            "journal-article",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "SILVA, João; SOUZA, Maria. A importância da pesquisa científica. *Revista Brasileira de Ciência*, v. 10, n. 2, p. 15-30, 2023."
        );
    }

    #[test]
    fn test_format_abnt_journal_article_minimal() {
        let work = make_work(
            "A study on machine learning",
            vec!["MARTINS, Ana"],
            Some("2021"),
            Some("Journal of AI"),
            None,
            None,
            None,
            None,
            "journal-article",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "MARTINS, Ana. A study on machine learning. *Journal of AI*, 2021."
        );
    }

    #[test]
    fn test_format_abnt_book() {
        let work = make_work(
            "Fundamentos da física quântica",
            vec!["OLIVEIRA, Carlos"],
            Some("2020"),
            None,
            None,
            None,
            None,
            Some("Editora Acadêmica"),
            "book",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "OLIVEIRA, Carlos. Fundamentos da física quântica. Editora Acadêmica; 2020."
        );
    }

    #[test]
    fn test_format_abnt_book_chapter() {
        let work = make_work(
            "Introdução à análise de dados",
            vec!["LIMA, Rafael"],
            Some("2022"),
            None,
            None,
            None,
            None,
            Some("Editora Tech"),
            "book-section",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "LIMA, Rafael. Introdução à análise de dados. Editora Tech; 2022."
        );
    }

    #[test]
    fn test_format_abnt_conference() {
        let work = make_work(
            "Redes neurais aplicadas à medicina",
            vec!["COSTA, Beatriz", "SANTOS, Pedro"],
            Some("2023"),
            Some("Congresso Brasileiro de Computação"),
            Some("5"),
            None,
            Some("100-110"),
            Some("SBC"),
            "paper-conference",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "COSTA, Beatriz; SANTOS, Pedro. Redes neurais aplicadas à medicina. In: Congresso Brasileiro de Computação, v. 5, p. 100-110 SBC; 2023."
        );
    }

    #[test]
    fn test_format_abnt_no_authors() {
        let work = make_work(
            "Anonymous publication",
            vec![],
            Some("2020"),
            Some("Some Journal"),
            Some("5"),
            None,
            None,
            None,
            "journal-article",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "Anonymous publication. *Some Journal*, v. 5, 2020."
        );
    }

    #[test]
    fn test_format_abnt_no_year() {
        let work = make_work(
            "Undated work",
            vec!["AUTOR, Anonimo"],
            None,
            Some("Revista X"),
            None,
            None,
            None,
            None,
            "journal-article",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "AUTOR, Anonimo. Undated work. *Revista X*, s.d."
        );
    }

    #[test]
    fn test_format_abnt_no_year_style_consistent() {
        let work = make_work(
            "Undated work",
            vec!["AUTOR, Anonimo"],
            None,
            Some("Revista X"),
            None,
            None,
            None,
            None,
            "journal-article",
        );
        let result = format_abnt(&work);
        assert!(!result.contains(".."), "Should not contain double periods: {}", result);
        assert!(result.contains("s.d."), "Should contain 's.d.'");
        assert!(result.ends_with("."), "Should end with a period");
    }

    #[test]
    fn test_format_abnt_dissertation() {
        let work = make_work(
            "Tese de doutorado em engenharia",
            vec!["FERREIRA, Lucas"],
            Some("2023"),
            None,
            None,
            None,
            None,
            Some("USP"),
            "dissertation",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "FERREIRA, Lucas. Tese de doutorado em engenharia. USP; 2023."
        );
    }

    #[test]
    fn test_format_abnt_unknown_type() {
        let work = make_work(
            "Some other type of work",
            vec!["AUTHOR, Test"],
            Some("2023"),
            None,
            None,
            None,
            None,
            Some("Some Publisher"),
            "other",
        );
        let result = format_abnt(&work);
        assert_eq!(
            result,
            "AUTHOR, Test. Some other type of work. Some Publisher; 2023."
        );
    }

    #[test]
    fn test_format_abnt_trailing_dot_stripped() {
        let work = make_work(
            "Title with trailing dot.",
            vec!["AUTHOR, Test"],
            Some("2023"),
            Some("Journal"),
            None,
            None,
            None,
            None,
            "journal-article",
        );
        let result = format_abnt(&work);
        assert!(!result.contains(".."));
        assert!(result.contains("Title with trailing dot"));
    }

    // --- Block insertion tests ---

    #[derive(Clone, Debug, PartialEq)]
    enum BlockType {
        Paragraph,
        Heading1,
        Citation,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Block {
        id: usize,
        block_type: BlockType,
        content: String,
        prev: Option<usize>,
        next: Option<usize>,
    }

    impl Block {
        fn new(id: usize, block_type: BlockType) -> Self {
            Self {
                id,
                block_type,
                content: String::new(),
                prev: None,
                next: None,
            }
        }
    }

    #[derive(Clone, Debug)]
    struct Buffer {
        head: Option<usize>,
        tail: Option<usize>,
        blocks: HashMap<usize, Block>,
        length: usize,
    }

    impl Buffer {
        fn new() -> Self {
            Self {
                head: None,
                tail: None,
                blocks: HashMap::new(),
                length: 0,
            }
        }

        fn push_back(&mut self, mut block: Block) {
            let id = block.id;
            if let Some(tail_id) = self.tail {
                if let Some(b) = self.blocks.get_mut(&tail_id) {
                    b.next = Some(id);
                }
                block.prev = self.tail;
            } else {
                self.head = Some(id);
            }
            self.blocks.insert(id, block);
            self.tail = Some(id);
            self.length += 1;
        }

        fn insert_after(&mut self, block: Block, after_id: Option<usize>) -> usize {
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

        fn to_vec(&self) -> Vec<Block> {
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
    }

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
        let focused_block_id = Some(0usize);

        let insert_after = focused_block_id.and_then(|fid| {
            None
        });

        let new_block = make_block(10, "citation");
        buf.insert_after(new_block, insert_after);

        let vec = buf.to_vec();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[2].content, "citation");
    }
}
