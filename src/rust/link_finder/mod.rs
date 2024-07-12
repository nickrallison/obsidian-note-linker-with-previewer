use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

use crate::parser::ParsedMDFile;
use crate::prelude::*;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) struct Link {
    pub source: PathBuf,
    pub target: PathBuf,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl Link {
    pub(crate) fn new(
        source: PathBuf,
        target: PathBuf,
        byte_start: usize,
        byte_end: usize,
    ) -> Self {
        Link {
            source,
            target,
            byte_start,
            byte_end,
        }
    }
    pub(crate) fn ser(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub(crate) fn deser(s: &str) -> Self {
        serde_json::from_str(s).unwrap()
    }
}

#[derive(Debug)]
pub(crate) struct LinkFinder {
    groups: Vec<(PathBuf, String)>,
    settings: crate::settings::Settings,
}

impl LinkFinder {
    pub(crate) fn new(
        files: Vec<&crate::vault::File>,
        settings: crate::settings::Settings,
    ) -> Self {
        let mut file_groups: HashMap<usize, PathBuf> = HashMap::new();
        let mut group_index: usize = 1;
        let mut file_regex_strs: Vec<(PathBuf, String)> = vec![];

        for file in files {
            let aliases: Vec<&str> = file.get_aliases();
            // escape all regex special characters
            let cleaned_aliases: Vec<String> =
                aliases.iter().map(|alias| regex::escape(alias)).collect();

            let mut file_regex_str: String = String::new();
            file_regex_str.push('(');
            for alias in cleaned_aliases {
                file_regex_strs.push((file.path.clone(), format!("(\\b{}\\b)", alias)));
            }
        }

        file_regex_strs.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        file_regex_strs.reverse();

        LinkFinder {
            groups: file_regex_strs,
            settings: settings,
        }
    }

    pub(crate) fn create_regex(&self) -> Result<(Regex, HashMap<usize, PathBuf>)> {
        let mut file_groups: HashMap<usize, PathBuf> = HashMap::new();
        let mut group_index: usize = 1;
        for (path, regex_str) in self.groups.iter() {
            file_groups.insert(group_index, path.clone());
            group_index += 1;
        }
        let regex_strs: Vec<String> = self
            .groups
            .iter()
            .map(|(_, regex_str)| regex_str.to_string())
            .collect::<Vec<String>>();
        let regex_str: String = regex_strs.join("|");
        let regex: Regex = RegexBuilder::new(&regex_str)
            .case_insensitive(self.settings.case_insensitive)
            .build()?;
        Ok((regex, file_groups))
    }

    pub(crate) fn create_regex_exc(
        &self,
        path: &PathBuf,
    ) -> Result<(Regex, HashMap<usize, PathBuf>)> {
        let groups: Vec<(PathBuf, String)> = self
            .groups
            .iter()
            .filter(|(p, _)| p != path)
            .map(|(p, s)| (p.clone(), s.clone()))
            .collect();
        let mut file_groups: HashMap<usize, PathBuf> = HashMap::new();
        let mut group_index: usize = 1;
        for (path, regex_str) in groups.iter() {
            file_groups.insert(group_index, path.clone());
            group_index += 1;
        }
        let regex_strs: Vec<String> = groups
            .iter()
            .map(|(_, regex_str)| regex_str.to_string())
            .collect::<Vec<String>>();
        let regex_str: String = regex_strs.join("|");
        let regex: Regex = RegexBuilder::new(&regex_str)
            .case_insensitive(self.settings.case_insensitive)
            .build()?;
        Ok((regex, file_groups))
    }

    pub(crate) fn get_links(&self, md_file: &crate::vault::File) -> Vec<Link> {
        let md_file: &ParsedMDFile = &md_file.contents;
        let mut links: Vec<Link> = vec![];
        let (regex, group_map) = self.create_regex_exc(&md_file.path).unwrap();
        let num_groups = group_map.len();
        let string_nodes: Vec<crate::parser::Node> = md_file.get_string_nodes();

        for node in string_nodes {
            let start: usize = node.start;
            let end: usize = node.end;
            let string: Result<&str> = node.get_inner_string();
            match string {
                Ok(string) => {
                    let caps_iter: regex::CaptureMatches<'_, '_> = regex.captures_iter(string);
                    // println!("caps_iter:{:?}", caps_iter);
                    for caps in caps_iter {
                        let cap_result: Option<(regex::Match, usize)> =
                            get_first_capture(Some(caps), num_groups);
                        match cap_result {
                            Some((capture, group_index)) => {
                                let capture_str: &str = capture.as_str();
                                let cap_start = capture.start();
                                let target: &Path =
                                    group_map.get(&group_index).expect("expected group");
                                let source: &Path = &md_file.path;
                                let link_text: &str = capture_str;
                                let link: Link = Link {
                                    source: source.to_path_buf(),
                                    target: target.to_path_buf(),
                                    byte_start: start + cap_start,
                                    byte_end: start + cap_start + link_text.len(),
                                };
                                links.push(link);
                            }
                            None => (),
                        }
                    }
                }
                Err(_) => (),
            }
        }
        if !self.settings.link_to_self {
            links.retain(|link| link.source != link.target);
        }
        links
    }
}

fn get_first_capture(
    caps: Option<regex::Captures>,
    caps_len: usize,
) -> Option<(regex::Match, usize)> {
    match caps {
        Some(captures) => {
            for i in 1..caps_len + 1 {
                let i: usize = i as usize;
                if captures.get(i).is_some() {
                    return Some((
                        captures.get(i).expect("Expected capture to exist."),
                        i as usize,
                    ));
                }
            }
        }
        None => (),
    }
    None
}

const FILE_1_PATH: &str = "alan turing.md";
const FILE_1_CONT: &str = r#"---
bad_links: 
tags: [computerscience]
date created: Monday, July 10th 2023, 12:23:57 am
title: Alan Turing
aliases: ["turing"]
---

# Alan Turing

Alan Mathison Turing, the father of the Turing Machine was born on June 23, 1912, in London. He is widely considered as the father of theoretical computer science and artificial intelligence. Turing studied mathematics at King's College, University of Cambridge, where he developed the concept of a "universal machine" that could compute anything that is computable. This idea formed the basis of all modern computers.  
During World War II, Turing worked at Bletchley Park, Britain's codebreaking centre, and was instrumental in breaking the German Enigma code. His work is said to have significantly shortened the war and saved countless lives.  
Post war, Turing worked on developing an early computer at the National Physical Laboratory and later on artificial intelligence at the University of Manchester. He proposed an experiment now known as the "Turing Test" to determine if a machine can exhibit intelligent behavior equivalent to or indistinguishable from human behavior.  
Despite his accomplishments, Turing faced persecution for his homosexuality - which was illegal in Britain at that time. He was convicted for "gross indecency" in 1952 and underwent chemical castration as an alternative to prison. Tragically, this led to his untimely death by suicide on June 7, 1954.  
In 2013, Queen Elizabeth II granted Turing a posthumous royal pardon. His legacy continues through the "Turing Award", often referred to as the 'Nobel Prize' of computing world which is given annually by ACM (Association for Computing Machinery). In addition, his life and work have been depicted in various forms including the 2014 film "The Imitation Game"."#;

const FILE_2_PATH: &str = "turing machine.md";
const FILE_2_CONT: &str = r#"---
bad_links: 
aliases: []
tags: [computerscience, theoreticalcompsci]
title: Turing Machine
date created: Monday, July 24th 2023, 7:44:20 pm
---
# Turing Machine

A Turing Machine is a theoretical computational device, conceived by British mathematician Alan Turing in 1936. Its an abstract model of computation that manipulates symbols on a strip of tape according to a table of rules. Despite its simplicity, a Turing machine can simulate the logic of any computer algorithm and is used in theoretical computer science to understand what can be computed. Its also a key concept in the theory of computation and computability."#;

#[cfg(test)]
pub mod link_finder_test {
    use include_dir::{include_dir, Dir};
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn regex_construct_test() {
        let settings = crate::settings::Settings::new(true, false, "red".to_string());
        let file_1_path: PathBuf = PathBuf::from(FILE_1_PATH);
        let file_2_path: PathBuf = PathBuf::from(FILE_2_PATH);
        let file1 = crate::vault::File::new(file_1_path, FILE_1_CONT.to_string()).unwrap();
        let file2 = crate::vault::File::new(file_2_path, FILE_2_CONT.to_string()).unwrap();
        let files = vec![&file1, &file2];
        let link_finder = LinkFinder::new(files, settings);
        let (regex, group_map) = link_finder.create_regex().unwrap();

        assert_eq!(group_map.len(), 3);
        assert_eq!(group_map.get(&1).unwrap(), &PathBuf::from(FILE_2_PATH));
        assert_eq!(group_map.get(&2).unwrap(), &PathBuf::from(FILE_1_PATH));
        assert_eq!(group_map.get(&2).unwrap(), &PathBuf::from(FILE_1_PATH));
        assert_eq!(
            regex.as_str(),
            r#"(\bturing machine\b)|(\balan turing\b)|(\bturing\b)"#
        );
    }
    #[test]
    fn link_alan_turing_test() {
        let settings = crate::settings::Settings::new(true, false, "red".to_string());
        let file_1_path: PathBuf = PathBuf::from(FILE_1_PATH);
        let file_2_path: PathBuf = PathBuf::from(FILE_2_PATH);
        let file1 = crate::vault::File::new(file_1_path, FILE_1_CONT.to_string()).unwrap();
        let file2 = crate::vault::File::new(file_2_path, FILE_2_CONT.to_string()).unwrap();
        let files = vec![&file1, &file2];
        let link_finder = LinkFinder::new(files, settings);

        let (regex, group_map) = link_finder
            .create_regex_exc(&PathBuf::from(FILE_1_PATH))
            .unwrap();

        assert_eq!(group_map.len(), 1);
        assert_eq!(group_map.get(&1).unwrap(), &PathBuf::from(FILE_2_PATH));
        assert_eq!(regex.as_str(), r#"(\bturing machine\b)"#);

        let links: Vec<Link> = link_finder.get_links(&file1);

        let links_expected: Vec<Link> = vec![Link {
            source: PathBuf::from(FILE_1_PATH),
            target: PathBuf::from(FILE_2_PATH),
            byte_start: 189,
            byte_end: 203,
        }];
        assert_eq!(links, links_expected);
    }

    #[test]
    fn link_turing_machine_test() {
        let settings = crate::settings::Settings::new(true, false, "red".to_string());
        let file_1_path: PathBuf = PathBuf::from(FILE_1_PATH);
        let file_2_path: PathBuf = PathBuf::from(FILE_2_PATH);
        let file1 = crate::vault::File::new(file_1_path, FILE_1_CONT.to_string()).unwrap();
        let file2 = crate::vault::File::new(file_2_path, FILE_2_CONT.to_string()).unwrap();
        let files = vec![&file1, &file2];
        let link_finder = LinkFinder::new(files, settings);
        let (regex, group_map) = link_finder
            .create_regex_exc(&PathBuf::from(FILE_2_PATH))
            .unwrap();

        assert_eq!(group_map.len(), 2);
        assert_eq!(group_map.get(&1).unwrap(), &PathBuf::from(FILE_1_PATH));
        assert_eq!(group_map.get(&2).unwrap(), &PathBuf::from(FILE_1_PATH));
        assert_eq!(regex.as_str(), r#"(\balan turing\b)|(\bturing\b)"#);

        let links: Vec<Link> = link_finder.get_links(&file2);
        /*
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 149, byte_end: 155 }
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 167, byte_end: 173 }
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 256, byte_end: 267 }
           Link { source: "turing machine.md", target: "alan turing.md", byte_start: 415, byte_end: 421 }
        */
        let links_expected: Vec<Link> = vec![
            Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 149,
                byte_end: 155,
            },
            Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 167,
                byte_end: 173,
            },
            Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 256,
                byte_end: 267,
            },
            Link {
                source: PathBuf::from(FILE_2_PATH),
                target: PathBuf::from(FILE_1_PATH),
                byte_start: 415,
                byte_end: 421,
            },
        ];
        assert_eq!(links, links_expected);
    }
    // #[test]
    // fn link_finder_coverage_test() {
    //     static PROJECT_DIR: Dir<'_> = include_dir!("test");
    //     let settings = crate::settings::Settings::new(true, false, "red".to_string());
    //     let mut files: Vec<Result<crate::vault::File>> = vec![];
    //     for file in PROJECT_DIR.files() {
    //         let path = file.path().to_path_buf();
    //         let content = file.contents_utf8().unwrap().to_string();
    //         files.push(crate::vault::File::new(path, content));
    //     }
    //     let mut valid_files: Vec<crate::vault::File> = vec![];
    //     for file in files {
    //         match file {
    //             Ok(file) => valid_files.push(file),
    //             Err(_) => (),
    //         }
    //     }
    //     let link_finder = LinkFinder::new(valid_files.iter().collect(), settings);
    // }
}
