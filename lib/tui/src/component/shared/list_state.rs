use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug)]
pub struct ListState {
    options: Vec<String>,
    sorted_options: Vec<usize>,
    max_rows: usize,
    index: usize,
    index_offset: usize,
}

impl ListState {
    pub fn new(options: Vec<String>) -> Self {
        let sorted_options = (0..options.len()).collect();

        Self {
            options,
            sorted_options,
            max_rows: 6,
            index: 0,
            index_offset: 0,
        }
    }

    pub fn next(&mut self) {
        if self.index == self.sorted_options.len().saturating_sub(1) {
            self.index_offset = 0;
        } else if self.index == self.index_offset.saturating_add(self.max_rows.saturating_sub(1)) {
            self.index_offset = self.index_offset.saturating_add(1);
        }

        self.index = self.index.saturating_add(1) % self.sorted_options.len();
    }

    pub fn prev(&mut self) {
        if self.index == 0 {
            self.index_offset = self.sorted_options.len().saturating_sub(self.max_rows);
        } else if self.index == self.index_offset {
            self.index_offset = self.index_offset.saturating_sub(1);
        }

        self.index = self.index.saturating_add(self.sorted_options.len().saturating_sub(1)) % self.sorted_options.len();
    }

    pub fn sort(&mut self, pattern: &str) {
        self.index = 0;
        self.index_offset = 0;

        if pattern.is_empty() {
            self.sorted_options = (0..self.options.len()).collect();
            return;
        }

        let matcher = SkimMatcherV2::default();
        let mut scores = vec![];

        for i in 0..self.options.len() {
            if let Some(score) = matcher.fuzzy_match(&self.options[i], pattern) {
                scores.push((i, score));
            }
        }

        scores.sort_by(|a, b| b.1.cmp(&a.1));

        self.sorted_options.clear();
        for (i, _) in scores {
            self.sorted_options.push(i);
        }
    }

    pub fn clear(&mut self) {
        self.index = 0;
        self.index_offset = 0;
        self.sorted_options.clear();
    }

    /// An index into the sorted options of the list
    pub fn index(&self) -> Option<usize> {
        if self.sorted_options.is_empty() {
            return None;
        }

        Some(self.index.saturating_sub(self.index_offset))
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = self.index_offset.saturating_add(index);
    }

    pub fn selection(&self) -> Option<&str> {
        let index = self.index()?;
        Some(
            self.options
                .get(*self.sorted_options.get(index.saturating_add(self.index_offset))?)?,
        )
    }

    pub fn options(&self) -> &Vec<String> {
        &self.options
    }

    pub fn sorted_options(&self) -> Vec<&str> {
        let mut sorted_options = vec![];
        for &option in
            &self.sorted_options[self.index_offset..self.sorted_options.len().min(self.index_offset + self.max_rows)]
        {
            sorted_options.push(self.options[option].as_str())
        }

        sorted_options
    }

    pub fn max_rows(&self) -> usize {
        self.max_rows
    }
}
