use vte::Params;

// INFO: Utility functions for unpacking and indexing ANSI CSI escape parameter sequences.
pub fn flatten_params(params: &Params) -> Vec<u16> {
    let mut out = Vec::new();
    for group in params.iter() {
        if group.is_empty() {
            out.push(0);
        } else {
            out.extend_from_slice(group);
        }
    }
    out
}

pub fn first_param(params: &Params, default: u16) -> u16 {
    match params.iter().next() {
        Some(g) if !g.is_empty() && g[0] != 0 => g[0],
        _ => default,
    }
}

pub fn nth_param(params: &Params, n: usize, default: u16) -> u16 {
    match params.iter().nth(n) {
        Some(g) if !g.is_empty() && g[0] != 0 => g[0],
        _ => default,
    }
}
