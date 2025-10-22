use crate::embedding::tokenize;
use regex::Regex;

pub fn split_paragraphs(input: &str) -> Vec<String> {
    // Split on blank lines as paragraphs
    let mut paras: Vec<String> = input
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if paras.is_empty() {
        paras.push(input.trim().to_string());
    }
    paras
}

pub fn split_sentences(input: &str) -> Vec<String> {
    // A simple sentence splitter; not perfect but OK for demo
    let re = Regex::new(r"(?s)(.*?[\.\!\?])\s+").unwrap();
    let mut out = Vec::new();
    let mut last = 0usize;
    for cap in re.captures_iter(input) {
        if let Some(m) = cap.get(1) {
            out.push(m.as_str().trim().to_string());
            last = m.end();
        }
    }
    let tail = &input[last..];
    if !tail.trim().is_empty() {
        out.push(tail.trim().to_string());
    }
    out
}

pub fn chunk_by_sentences(sentences: &[String], window: usize) -> Vec<String> {
    if window == 0 {
        return vec![];
    }
    let mut chunks = Vec::new();
    let mut i = 0usize;
    while i < sentences.len() {
        let end = usize::min(i + window, sentences.len());
        let chunk = sentences[i..end].join(" ");
        chunks.push(chunk);
        i = end;
    }
    chunks
}

pub fn chunk_text(input: &str) -> Vec<String> {
    let paras = split_paragraphs(input);
    if paras.len() >= 5 {
        // enough paragraphs
        return paras;
    }
    let sents = split_sentences(input);
    let window = if sents.len() > 50 { 8 } else { 5 };
    chunk_by_sentences(&sents, window)
}

pub fn chunk_by_token_overlap(input: &str, tokens_per_chunk: usize, overlap: usize) -> Vec<String> {
    let toks = tokenize(input);
    if tokens_per_chunk == 0 {
        return vec![];
    }
    let step = if tokens_per_chunk > overlap {
        tokens_per_chunk - overlap
    } else {
        1
    };
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < toks.len() {
        let end = usize::min(i + tokens_per_chunk, toks.len());
        let chunk = toks[i..end].join(" ");
        out.push(chunk);
        if end == toks.len() {
            break;
        }
        i += step;
    }
    out
}
