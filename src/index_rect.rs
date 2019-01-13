// Copyright 2016-2018 Matthew D. Michelotti
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// IndexRect represents a non-empty rectangular index range in a 2-D grid.

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct IndexRect {
    start: (i32, i32),
    end: (i32, i32),
}

impl IndexRect {
    // start is inclusive, end is exclusive
    pub fn new(start: (i32, i32), end: (i32, i32)) -> IndexRect {
        assert!(
            start.0 < end.0 && start.1 < end.1,
            "IndexRect contains no elements"
        );
        IndexRect { start, end }
    }

    pub fn iter(self) -> Iter {
        Iter::new(self)
    }

    pub fn contains(self, val: (i32, i32)) -> bool {
        val.0 >= self.start.0 && val.0 < self.end.0 && val.1 >= self.start.1 && val.1 < self.end.1
    }
}

pub struct Iter {
    rect: IndexRect,
    val: Option<(i32, i32)>,
}

impl Iter {
    fn new(rect: IndexRect) -> Iter {
        Iter { rect, val: None }
    }
}

impl Iterator for Iter {
    type Item = (i32, i32);
    fn next(&mut self) -> Option<(i32, i32)> {
        self.val = match self.val {
            Some((x, y)) => {
                if y == self.rect.end.1 - 1 {
                    if x == self.rect.end.0 - 1 {
                        None
                    } else {
                        Some((x + 1, self.rect.start.1))
                    }
                } else {
                    Some((x, y + 1))
                }
            }
            None => Some(self.rect.start),
        };
        self.val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_iterator() {
        let rect = IndexRect::new((2, 3), (5, 7));
        let mut set = HashSet::new();
        for (x, y) in rect.iter() {
            assert!(x >= 2 && x < 5);
            assert!(y >= 3 && y < 7);
            assert!(set.insert((x, y)));
        }
        assert_eq!(set.len(), 12);
    }

    #[test]
    fn test_contains() {
        let rect = IndexRect::new((2, 3), (5, 7));
        assert!(rect.contains((2, 3)));
        assert!(rect.contains((4, 6)));
        assert!(!rect.contains((1, 3)));
        assert!(!rect.contains((2, 2)));
        assert!(!rect.contains((5, 6)));
        assert!(!rect.contains((4, 7)));
    }

    #[test]
    #[should_panic]
    fn test_new_bad_x() {
        IndexRect::new((4, -5), (4, -4));
    }

    #[test]
    #[should_panic]
    fn test_new_bad_y() {
        IndexRect::new((4, -5), (5, -5));
    }

    #[test]
    fn test_new() {
        IndexRect::new((4, -5), (5, -4));
    }
}
