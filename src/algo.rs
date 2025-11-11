use crate::Password;

// Insertion Sort
pub fn sort(list: &mut Vec<Password>) {
    let len = list.len();
    for i in 0..len {
        for j in 0..i {
            if alphabetical(&list[j].name, &list[i].name) {
                let password = list.remove(i);
                list.insert(j, password);
            }
        }
    }
}

// Compare two strings alphabetically
fn alphabetical(lhs: &str, rhs: &str) -> bool {
    let mut lhs = lhs.chars();
    let mut rhs = rhs.chars();

    loop {
        let Some(l) = lhs.next() else {
            return false;
        };

        let Some(r) = rhs.next() else {
            return true;
        };

        if l > r {
            return true;
        } else if l < r {
            return false;
        }
    }
}

// Binary Search
pub fn contains(name: &str, list: &[Password]) -> bool {
    let mut lhs = 0;
    let mut rhs = list.len();

    loop {
        let middle = (rhs - lhs) / 2 + lhs;
        if middle == lhs {
            return false;
        }

        let element = &list[middle];

        if name == element.name {
            return true;
        } else if alphabetical(name, &element.name) {
            lhs = middle;
        } else {
            rhs = middle;
        }
    }
}
