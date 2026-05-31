#!/usr/bin/env python3
"""
Fix bare underscores in Markdown prose for GitHub LaTeX rendering.
'_' is only allowed inside backticks or inside $...$ / $$...$$ math mode.
"""

import os
import re

BASE = r"c:\Users\nl\Dropbox\kelvin\documentation\formal_verification"

FILES = [
    "C1/gap1_kop_shannon_bound.md",
    "C1/gap2_uniform_distribution.md",
    "C1/gap3_saturation_bound.md",
    "C1/gap4_epsilon_bound.md",
    "C1/gap5_error_independence.md",
    "C1/gap6_hartley_min_entropy.md",
    "C1/gap7_verlet_double_effect.md",
    "C1/proof_sketch.md",
    "C2/proof_sketch.md",
    "C2/gap1_discrete_lyapunov_bound.md",
    "C2/gap2_positive_lambda_certification.md",
    "C2/gap3_q3264_lyapunov_spectrum.md",
    "C2/gap4_kaplan_yorke_entropy.md",
    "C2/gap5_shadow_orbit_bias.md",
    "C2/gap6_lipschitz_lambda.md",
    "C3/proof_sketch.md",
    "C3/gap1_ambainis_adversary.md",
    "C3/gap2_quantum_query_lower_bound.md",
    "C3/gap3_c1_c3_link.md",
    "C4/proof_sketch.md",
    "C4/gap1_game_based_reduction.md",
    "C4/gap2_domain_separation_all_modes.md",
    "C4/gap3_advantage_bound.md",
    "C5/proof_sketch.md",
    "C5/gap1_cardinality_bound.md",
    "C5/gap2_min_entropy.md",
    "C5/gap3_stability_reduction.md",
    "C5/gap4_symbolic_kani_config.md",
    "code_verification.md",
    "README.md",
]


def get_word_range(line, pos):
    """Get the start and end of the word containing position pos."""
    # Walk left to find start of word (non-word chars)
    start = pos
    while start > 0 and is_word_char(line[start - 1]):
        start -= 1
    # Walk right to find end of word
    end = pos + 1
    while end < len(line) and is_word_char(line[end]):
        end += 1
    return start, end


def is_word_char(c):
    """Check if character can be part of an identifier (including Greek letters)."""
    if c.isalnum():
        return True
    # Also allow Greek letters (Unicode range)
    cp = ord(c)
    if 0x03B1 <= cp <= 0x03C9:  # Greek lowercase
        return True
    if 0x0391 <= cp <= 0x03A9:  # Greek uppercase
        return True
    if c == '_':  # underscore is a word char for identifiers
        return True
    if c == '{' or c == '}':  # LaTeX braces are part of \text{} expressions
        return True
    return False


def fix_file(filepath):
    with open(filepath, "r", encoding="utf-8") as f:
        text = f.read()
    
    lines = text.split("\n")
    result = []
    in_code_block = False
    in_math_block = False
    
    for line in lines:
        stripped = line.strip()
        
        # Track ``` code blocks
        if stripped.startswith("```"):
            in_code_block = not in_code_block
            result.append(line)
            continue
        
        # Track $$ math blocks
        if stripped.startswith("$$"):
            in_math_block = not in_math_block
            result.append(line)
            continue
        
        if in_code_block or in_math_block:
            result.append(line)
            continue
        
        # Process line for bare underscores
        # Find all '_' that are NOT inside backticks
        bt_open = -1  # position of opening backtick, -1 if not inside backtick span
        dollar_open = -1  # position of opening $, -1 if not inside inline math
        positions_to_fix = []
        
        i = 0
        while i < len(line):
            ch = line[i]
            
            # Track backtick spans
            if ch == '`':
                # Skip ``` blocks (already handled at line level, but also check inline)
                if i + 2 < len(line) and line[i:i+3] == '```':
                    i += 3
                    continue
                if bt_open == -1:
                    bt_open = i
                else:
                    # Check if this is truly the closing backtick (not nested)
                    bt_open = -1
                i += 1
                continue
            
            # Track inline math $...$ (not $$)
            if ch == '$':
                if i + 1 < len(line) and line[i+1] == '$':
                    i += 2
                    continue
                if dollar_open == -1:
                    dollar_open = i
                else:
                    dollar_open = -1
                i += 1
                continue
            
            # Check for bare underscore
            if ch == '_' and bt_open == -1 and dollar_open == -1:
                positions_to_fix.append(i)
            
            i += 1
        
        # Fix positions from right to left to preserve positions
        for pos in reversed(positions_to_fix):
            start, end = get_word_range(line, pos)
            word = line[start:end]
            # Wrap in backticks if not already wrapped
            line = line[:start] + '`' + word + '`' + line[end:]
        
        result.append(line)
    
    with open(filepath, "w", encoding="utf-8") as f:
        f.write("\n".join(result))


def main():
    for rel_path in FILES:
        full_path = os.path.join(BASE, rel_path)
        if not os.path.exists(full_path):
            print(f"SKIP (not found): {rel_path}")
            continue
        print(f"Processing: {rel_path} ... ", end="", flush=True)
        fix_file(full_path)
        print("OK")
    
    print("\nAll done.")


if __name__ == "__main__":
    main()