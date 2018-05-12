use std::ffi::{CStr, CString};
use std::marker;
use std::slice;

use yara_sys;

use errors::*;
use internals::get_tidx;
use Match;
use Rule;
use YrString;

pub fn rules_destroy(rules: &mut yara_sys::YR_RULES) {
    unsafe {
        yara_sys::yr_rules_destroy(rules);
    }
}

// TODO Check if non mut
pub fn rules_save(rules: &mut yara_sys::YR_RULES, filename: &str) -> Result<(), YaraError> {
    let filename = CString::new(filename).unwrap();
    let result = unsafe { yara_sys::yr_rules_save(rules, filename.as_ptr()) };
    YaraErrorKind::from_yara(result)
}

impl<'a, 'b: 'a> From<&'a yara_sys::YR_RULE> for Rule<'b> {
    fn from(rule: &yara_sys::YR_RULE) -> Self {
        let identifier = unsafe { CStr::from_ptr(rule.__bindgen_anon_1.identifier) }
            .to_str()
            .unwrap();
        let strings = YrStringIterator::from(rule).map(YrString::from).collect();

        Rule {
            identifier,
            strings,
        }
    }
}

struct YrStringIterator<'a> {
    head: *const yara_sys::YR_STRING,
    _marker: marker::PhantomData<&'a yara_sys::YR_STRING>,
}

impl<'a> From<&'a yara_sys::YR_RULE> for YrStringIterator<'a> {
    fn from(rule: &'a yara_sys::YR_RULE) -> YrStringIterator<'a> {
        YrStringIterator {
            head: unsafe { rule.__bindgen_anon_4.strings },
            _marker: marker::PhantomData::default(),
        }
    }
}

impl<'a> Iterator for YrStringIterator<'a> {
    type Item = &'a yara_sys::YR_STRING;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_null() {
            return None;
        }

        let string = unsafe { &*self.head };

        if string.g_flags as u32 & yara_sys::STRING_GFLAGS_NULL != 0 {
            None
        } else {
            self.head = unsafe { self.head.offset(1) };
            Some(string)
        }
    }
}

pub struct MatchIterator<'a> {
    head: *const yara_sys::_YR_MATCH,
    _marker: marker::PhantomData<&'a yara_sys::_YR_MATCH>,
}

impl<'a> From<&'a yara_sys::YR_MATCHES> for MatchIterator<'a> {
    fn from(matches: &'a yara_sys::YR_MATCHES) -> MatchIterator<'a> {
        MatchIterator {
            head: unsafe { matches.__bindgen_anon_1.head },
            _marker: marker::PhantomData::default(),
        }
    }
}

impl<'a> Iterator for MatchIterator<'a> {
    type Item = &'a yara_sys::_YR_MATCH;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.head.is_null() {
            let m = unsafe { &*self.head };
            self.head = m.next;
            Some(m)
        } else {
            None
        }
    }
}

impl<'a> From<&'a yara_sys::_YR_MATCH> for Match {
    fn from(m: &yara_sys::_YR_MATCH) -> Self {
        Match {
            offset: m.offset as usize,
            match_length: m.match_length as usize,
            data: Vec::from(unsafe { slice::from_raw_parts(m.data, m.data_length as usize) }),
        }
    }
}

impl<'a, 'b: 'a> From<&'a yara_sys::YR_STRING> for YrString<'b> {
    fn from(string: &yara_sys::YR_STRING) -> Self {
        let identifier = unsafe { CStr::from_ptr(string.__bindgen_anon_1.identifier) }
            .to_str()
            .unwrap();
        let tidx = get_tidx();
        let matches = MatchIterator::from(&string.matches[tidx as usize])
            .map(Match::from)
            .collect();

        YrString {
            identifier,
            matches,
        }
    }
}
