use rustywind_core::sorter::{self, FinderRegex};

use crate::Formatter;

impl Formatter<'_> {
    pub fn tailwind_expr(&mut self, attr_value: String) {
        static OPTIONS: sorter::Options = sorter::Options {
            regex: FinderRegex::DefaultRegex,
            sorter: sorter::Sorter::DefaultSorter,
            allow_duplicates: true,
        };

        let sorted = sorter::sort_classes(&attr_value, &OPTIONS);
        self.printer.word(sorted);
    }
}
