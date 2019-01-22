use regex;

const TABLE_MARKUP: &str = "<table>";
const TABLE_MARKUP_END: &str = "</table>";
const TABLE_ROW_MARKUP: &str = "<tr>";
const TABLE_ROW_MARKUP_END: &str = "</tr>";
const TABLE_HEADER_MARKUP: &str = "<thead>";
const TABLE_HEADER_MARKUP_END: &str = "</thead>";
const TABLE_HEADER_CELL_MARKUP: &str = "<th>";
const TABLE_HEADER_CELL_MARKUP_END: &str = "</th>";
const TABLE_BODY_MARKUP: &str = "<tbody>";
const TABLE_BODY_MARKUP_END: &str = "</tbody>";
const TABLE_CELL_MARKUP: &str = "<td>";
const TABLE_CELL_MARKUP_END: &str = "</td>";

struct TableMarkupBuilder {
    markup: String,
    rows: u32,
}

impl TableMarkupBuilder {
    fn new() -> TableMarkupBuilder {
        TableMarkupBuilder {markup: String::from(TABLE_MARKUP), rows: 0}
    }
    
    fn complete(&mut self) -> String {
        self.markup.push_str(TABLE_ROW_MARKUP_END);
        self.markup.push_str(TABLE_BODY_MARKUP_END);
        self.markup.push_str(TABLE_MARKUP_END);
        return self.markup.to_owned();
    }
    
    fn add_row(&mut self) {
        if self.rows == 0 {
            self.markup.push_str(TABLE_HEADER_MARKUP);
            self.markup.push_str(TABLE_ROW_MARKUP);
        } else if self.rows == 1 {
            self.markup.push_str(TABLE_ROW_MARKUP_END);
            self.markup.push_str(TABLE_HEADER_MARKUP_END);
            self.markup.push_str(TABLE_BODY_MARKUP);
            self.markup.push_str(TABLE_ROW_MARKUP);
        } else {
            self.markup.push_str(TABLE_ROW_MARKUP_END);
            self.markup.push_str(TABLE_ROW_MARKUP);
        }
        self.rows += 1;
    }
    
    fn cell(&mut self, data: &str) {
        if self.rows == 1 {
            self.markup.push_str(TABLE_HEADER_CELL_MARKUP);
            self.markup.push_str(data);
            self.markup.push_str(TABLE_HEADER_CELL_MARKUP_END);
        } else {
            self.markup.push_str(TABLE_CELL_MARKUP);
            self.markup.push_str(data);
            self.markup.push_str(TABLE_CELL_MARKUP_END);
        }
    }
}

pub fn markdown_to_html_table(content: &str) -> String {
    let mut table_markup = TableMarkupBuilder::new();
    let by_two_or_more_spaces = regex::Regex::new(r"\s{2,}").unwrap();
    let lines = content.trim().lines();
    for line in lines.into_iter() {
        table_markup.add_row();
        for data in by_two_or_more_spaces.split(line) {
            table_markup.cell(data);
        }
    }
    let markup = table_markup.complete();
    return markup;
}