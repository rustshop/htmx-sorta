use std::{cmp, debug_assert, mem};

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
/// An ordering key that can always find a key between two other keys.
///
/// The trick here is to treat end of array as a value 127.5, so it's
/// always larger than 0x7f, but smaller than 0x80. This way it's always
/// possible to find a key between two other keys, which is handy for
/// sorting things in a database.
pub struct SortId(Vec<u8>);

impl SortId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn cmp_raw(s: &[u8], other: &[u8]) -> cmp::Ordering {
        let mut s = s.iter().copied();
        let mut o = other.iter().copied();

        loop {
            match (s.next(), o.next()) {
                (None, None) => return cmp::Ordering::Equal,
                (None, Some(o)) => {
                    return if 0x80 <= o {
                        cmp::Ordering::Less
                    } else {
                        cmp::Ordering::Greater
                    }
                }
                (Some(s), None) => {
                    return if s < 0x80 {
                        cmp::Ordering::Less
                    } else {
                        cmp::Ordering::Greater
                    }
                }
                (Some(s), Some(o)) => match s.cmp(&o) {
                    cmp::Ordering::Equal => continue,
                    cmp => return cmp,
                },
            }
        }
    }
}

impl From<Vec<u8>> for SortId {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Ord for SortId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Self::cmp_raw(&self.0, &other.0)
    }
}

impl PartialOrd for SortId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn sortid_cmp() {
    for (a, b, r) in [
        (vec![], vec![], cmp::Ordering::Equal),
        (vec![0x00], vec![0x00], cmp::Ordering::Equal),
        (vec![0x7f], vec![0x7f], cmp::Ordering::Equal),
        (vec![0x80], vec![0x80], cmp::Ordering::Equal),
        (vec![0xff], vec![0xff], cmp::Ordering::Equal),
        (vec![0x00], vec![], cmp::Ordering::Less),
        (vec![0x7f], vec![], cmp::Ordering::Less),
        (vec![0x80], vec![], cmp::Ordering::Greater),
        (vec![0xff], vec![], cmp::Ordering::Greater),
        (vec![0x010, 0x00], vec![0x010], cmp::Ordering::Less),
        (vec![0x010, 0x7f], vec![0x010], cmp::Ordering::Less),
        (vec![0x010, 0x80], vec![0x010], cmp::Ordering::Greater),
        (vec![0x010, 0xff], vec![0x010], cmp::Ordering::Greater),
        (vec![0x00], vec![0x00, 122], cmp::Ordering::Greater),
        (vec![228, 1], vec![227, 128], cmp::Ordering::Greater),
    ] {
        assert_eq!(SortId::from(a).cmp(&SortId::from(b)), r);
    }
}

impl SortId {
    #[allow(unused)]

    /// Get a `SortId` to insert element in front of all the elements, given
    /// `existing_first`.
    pub fn in_front(existing_first: Option<&SortId>) -> SortId {
        let Some(existing_first) = existing_first else {
            return SortId::from(vec![]);
        };

        let mut res = vec![];

        for e in existing_first.0.iter().copied() {
            if e == 0x00 {
                res.push(e);
            } else {
                res.push(e / 2);
                return SortId::from(res);
            }
        }

        res.push(0x40);

        SortId::from(res)
    }

    pub fn at_the_end(existing_last: Option<&SortId>) -> SortId {
        let Some(existing_last) = existing_last else {
            return SortId::from(vec![]);
        };

        let mut res = vec![];

        for e in existing_last.0.iter().copied() {
            if e == 0xff {
                res.push(e);
            } else {
                res.push(0x80 + e / 2);
                return SortId::from(res);
            }
        }

        res.push(0xa0);

        SortId::from(res)
    }

    pub fn between(a: &SortId, b: &SortId) -> SortId {
        let mut l_i = a.0.iter().copied();
        let mut h_i = b.0.iter().copied();

        let mut r = vec![];

        'outer: loop {
            let (l, h) = (l_i.next(), h_i.next());

            // since so far a and b were the same, it's OK to swap them now
            // and pretend it was like since the beginning
            let (l, h) = match (l, h) {
                (Some(l), Some(h)) if h < l => {
                    mem::swap(&mut l_i, &mut h_i);
                    (Some(h), Some(l))
                }
                (Some(l), None) if 0x80 <= l => {
                    mem::swap(&mut l_i, &mut h_i);
                    (None, Some(l))
                }
                (None, Some(h)) if h < 0x80 => {
                    mem::swap(&mut l_i, &mut h_i);
                    (Some(h), None)
                }
                lh => lh,
            };

            match (l, h) {
                (Some(l), Some(h)) if l == h => {
                    r.push(l);
                }
                (Some(l), Some(h)) => {
                    debug_assert_ne!(l, h);
                    let mid = ((l as u16 + h as u16) / 2) as u8;
                    if mid != l && mid != h {
                        r.push(mid);
                        break;
                    }

                    match (l_i.next(), h_i.next()) {
                        (Some(0xff), Some(0x00)) => {
                            // just pick the l-side to avoid more complex backtracking
                            r.push(l);
                            r.push(0xff);
                            for l in l_i.by_ref() {
                                if l == 0xff {
                                    r.push(0xff);
                                } else {
                                    // a: 0x01 0xff 0x10
                                    // r: 0x01 0xff 0x85
                                    // b: 0x02 0x00
                                    r.push(l / 2 + 0x80);
                                    break 'outer;
                                }
                            }
                            // a: 0x01 0xff
                            // r: 0x01 0xff 0x80
                            // b: 0x02 0x00
                            r.push(0xa0);
                            break;
                        }
                        (Some(0xff), Some(hh)) => {
                            // a: 0x01 0xff
                            // r: 0x02 0x40
                            // b: 0x02 0x80
                            r.push(h);
                            r.push(hh / 2);
                            break;
                        }
                        (Some(ll), Some(0x00)) => {
                            // a: 0x01 0x80
                            // r: 0x01 0xa0
                            // b: 0x02 0x00
                            r.push(l);
                            r.push(ll / 2 + 0x80);
                            break;
                        }
                        (None, Some(0x00)) => {
                            // a: 0x01
                            // r: 0x01 0xa0
                            // b: 0x02 0x00
                            r.push(l);
                            r.push(0xa0);
                            break;
                        }
                        (Some(0xff), None) => {
                            // a: 0x01 0xff
                            // r: 0x02 0x40
                            // b: 0x02
                            r.push(h);
                            r.push(0x40);
                            break;
                        }
                        (None, None) => {
                            // a: 0x01
                            // r: 0x02 0x00
                            // b: 0x02
                            r.push(h);
                            r.push(0x00);
                            break;
                        }
                        (Some(ll), None) => {
                            // a: 0x01 0x80
                            // r: 0x01 0xa0
                            // b: 0x02
                            r.push(l);
                            r.push(ll / 2 + 0x80);
                            break;
                        }
                        (None, Some(hh)) => {
                            // a: 0x01
                            // r: 0x02
                            // b: 0x02 0x80
                            r.push(h);
                            r.push(hh / 2);
                            break;
                        }
                        (Some(ll), Some(hh)) => {
                            debug_assert!(ll != 0xff);
                            debug_assert!(hh != 0x00);
                            // a: 0x01 0x80
                            // r: 0x02
                            // b: 0x02 0x80
                            if ll < 0x80 {
                                r.push(l);
                                r.push(ll / 2 + 0x80);
                            } else if 0x80 <= hh {
                                r.push(h);
                                r.push(hh / 2);
                            } else {
                                // TODO: this is not optimal
                                r.push(l);
                                r.push(0xff);
                            }
                            break;
                        }
                    }
                }
                (Some(l), None) => {
                    debug_assert!(l < 0x80);
                    if l == 0x7f {
                        // l: 0x7f ...
                        // a: 0x7f ...
                        // h:
                        r.push(l);
                        loop {
                            match l_i.next() {
                                Some(0xff) => {
                                    // l: 0x7f 0xff ..
                                    // a: 0x7f 0xff ..
                                    // h:
                                    r.push(0xff);
                                }
                                Some(l) => {
                                    // l: 0x7f 0xfe
                                    // a: 0x7f 0xff
                                    // h:
                                    r.push(l / 2 + 0x80);
                                    break 'outer;
                                }
                                None => {
                                    // l: 0x7f
                                    // a: 0x7f 0xa0
                                    // h:
                                    r.push(0xa0);
                                    break 'outer;
                                }
                            }
                        }
                    } else {
                        // l: 0x40
                        // a: 0x60
                        // h:
                        r.push(l / 2 + 0x40);
                        break;
                    }
                }
                (None, Some(h)) => {
                    debug_assert!(0x80 <= h);

                    if h == 0x80 {
                        // l:
                        // a: 0x80 ...
                        // h: 0x80 ...
                        r.push(h);

                        loop {
                            match h_i.next() {
                                Some(0x00) => {
                                    // l:
                                    // a: 0x80 0x00 ...
                                    // h: 0x80 0x00 ...
                                    r.push(0x00);
                                }
                                Some(l) => {
                                    // l:
                                    // a: 0x80 0x00
                                    // h: 0x80 0x01
                                    r.push(l / 2);
                                    break 'outer;
                                }
                                None => {
                                    // l:
                                    // a: 0x80 0x40
                                    // h: 0x80
                                    r.push(0x40);
                                    break 'outer;
                                }
                            }
                        }
                    } else {
                        // l:
                        // a: 0xa0
                        // h: 0xc0
                        r.push(h / 2 + 0x40);
                        break;
                    }
                }
                (None, None) => {
                    // if inputs equal, the result is equal too
                    // l: 0x01
                    // r: 0x01
                    // h: 0x01
                    break;
                }
            }
        }

        SortId(r)
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::*;

    fn midpoint_sorts_correctly_impl(a: SortId, b: SortId) -> bool {
        let r = SortId::between(&a, &b);
        match a.cmp(&b) {
            cmp::Ordering::Equal => r == a && r == b,
            cmp::Ordering::Less => a < r && r < b,
            cmp::Ordering::Greater => b < r && r < a,
        }
    }
    #[quickcheck]
    fn midpoint_sorts_correctly_quickcheck(a: Vec<u8>, b: Vec<u8>) {
        assert!(
            midpoint_sorts_correctly_impl(SortId::from(a.clone()), SortId(b.clone())),
            "{a:?}, {b:?}"
        );
    }

    #[test]
    fn midpoint_sorts_correctly_manual() {
        for (a, b) in [
            (vec![], vec![]),
            (vec![0x00], vec![0x00, 0x00]),
            (vec![0x00], vec![0x00, 0x00, 0x00]),
            (vec![0x80], vec![0x80]),
            (vec![0x00], vec![0x00]),
            (vec![0x00], vec![]),
            (vec![0x00], vec![0xff]),
            (vec![0x01, 0x0a], vec![0x02, 0xff]),
            (vec![0x01, 0x00], vec![0x02, 0x40]),
            (vec![0x01, 0xff], vec![0x02, 0x00]),
            (vec![0x01, 0xff, 0xff], vec![0x02, 0x00, 0x00]),
            (vec![0x01, 0xff], vec![0x02, 0x00, 0x00]),
            (vec![0x01, 0xff, 0xff], vec![0x02, 0x00]),
            (vec![0xff], vec![]),
            // found by the quickcheck in the past
            (vec![0x00], vec![0x00, 122]),
            (vec![228, 1], vec![227, 128]),
            (vec![0x00, 127], vec![0]),
            (vec![252, 128], vec![253, 128]),
            (vec![0, 128], vec![1, 0]),
        ] {
            assert!(
                midpoint_sorts_correctly_impl(SortId::from(a.clone()), SortId::from(b.clone()),),
                "{a:?}, {b:?}"
            );
        }
    }
}
