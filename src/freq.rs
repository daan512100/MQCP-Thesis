// Bestand: src/freq.rs
//! Helpers voor lange-termijn frequentiegeheugen (Sectie 3.5 van de paper).
//!
//! Elke vertex v heeft een teller gₙ(v) die bijhoudt hoeveel keer v is
//! toegevoegd of verwijderd uit de huidige oplossing S.
//! - Bij elke add(v) of remove(v) wordt gₙ(v) met 1 verhoogd.
//! - Als gₙ(v) daarna groter is dan |S|, worden alle gₙ(*) weer op 0 gezet.

use crate::solution::Solution;

/// Voegt v toe aan S, verhoogt gₙ(v), en reset alle gₙ(*) als gₙ(v) > |S|.
pub fn add_counted<'g>(sol: &mut Solution<'g>, v: usize, freq_mem: &mut Vec<usize>) {
    sol.add(v);
    freq_mem[v] = freq_mem[v].saturating_add(1);
    let k = sol.size();
    if freq_mem[v] > k {
        freq_mem.fill(0);
    }
}

/// Verwijdert v uit S, verhoogt gₙ(v), en reset alle gₙ(*) als gₙ(v) > |S|.
pub fn remove_counted<'g>(sol: &mut Solution<'g>, v: usize, freq_mem: &mut Vec<usize>) {
    sol.remove(v);
    freq_mem[v] = freq_mem[v].saturating_add(1);
    let k = sol.size();
    if freq_mem[v] > k {
        freq_mem.fill(0);
    }
}
