use crate::types::{Instruction, OpCode, Operator};
use std::collections::HashSet;

pub fn optimize(code: Vec<Instruction>) -> Vec<Instruction> {
    let mut current_code = code;
    let mut pass = 1;
    loop {
        let (optimized_code, changed) = optimize_pass(&current_code);
        if !changed {
            break;
        }
        // println!("Optimization pass {} finished. Size: {} -> {}", pass, current_code.len(), optimized_code.len());
        current_code = optimized_code;
        pass += 1;
    }
    current_code
}

fn optimize_pass(code: &[Instruction]) -> (Vec<Instruction>, bool) {
    let mut new_code: Vec<(Instruction, usize)> = Vec::with_capacity(code.len());
    let mut changed = false;

    // 1. Identify jump targets
    let mut targets = HashSet::new();
    for instr in code {
        if instr.f == OpCode::JMP || instr.f == OpCode::JPC {
            targets.insert(instr.a as usize);
        }
    }

    // 2. Process instructions
    let mut i = 0;
    while i < code.len() {
        let instr = code[i];
        let mut pushed = false;

        if instr.f == OpCode::OPR {
            if let Some(op) = Operator::from_i64(instr.a) {
                // Check for algebraic simplifications with top of stack (last in new_code)
                if let Some(&(prev_instr, prev_idx)) = new_code.last() {
                    if prev_instr.f == OpCode::LIT {
                        let val = prev_instr.a;

                        // Check if we can optimize LIT val, OPR op
                        // Conditions: instr (OPR) and prev_instr (LIT) must not be jump targets
                        // Actually, prev_instr CAN be a jump target if we just remove the OPR (e.g. + 0)
                        // Wait, if we have `LIT 0, ADD`. `LIT 0` pushes 0. `ADD` pops 2, adds, pushes.
                        // Result is same as before `LIT 0`.
                        // So `LIT 0, ADD` is identity on the stack (except for the other operand).
                        // If we remove `LIT 0` and `ADD`, the stack is unchanged.
                        // If `LIT 0` is a jump target, and we jump there, we expect to push 0 then add.
                        // If we remove them, we land on whatever is after.
                        // If we jump to `LIT 0`, we expect `stack + 0`.
                        // If we remove them, we have `stack`.
                        // So it IS safe even if `LIT 0` is a jump target?
                        // YES, because the net effect of the block `LIT 0, ADD` is null.
                        // BUT, if we jump to `ADD` (index `i`), we expect `0` to be on stack (pushed by `LIT 0`).
                        // If we remove `LIT 0`, and jump to `ADD` (which is now gone), we land on next instr.
                        // But the stack is missing the `0`.
                        // So `i` (ADD) CANNOT be a jump target.
                        // `prev_idx` (LIT 0) CAN be a jump target.

                        let is_target_op = targets.contains(&i);

                        if !is_target_op {
                            match op {
                                Operator::ADD if val == 0 => {
                                    // LIT 0, ADD -> remove both
                                    new_code.pop();
                                    changed = true;
                                    pushed = true; // Handled (by dropping)
                                }
                                Operator::SUB if val == 0 => {
                                    // LIT 0, SUB -> remove both
                                    new_code.pop();
                                    changed = true;
                                    pushed = true;
                                }
                                Operator::MUL if val == 1 => {
                                    // LIT 1, MUL -> remove both
                                    new_code.pop();
                                    changed = true;
                                    pushed = true;
                                }
                                Operator::DIV if val == 1 => {
                                    // LIT 1, DIV -> remove both
                                    new_code.pop();
                                    changed = true;
                                    pushed = true;
                                }
                                _ => {}
                            }
                        }

                        if !pushed {
                            // Check for Constant Folding: LIT a, LIT b, OPR
                            if new_code.len() >= 2 {
                                let (prev2_instr, prev2_idx) = new_code[new_code.len() - 2];
                                if prev2_instr.f == OpCode::LIT {
                                    let val_a = prev2_instr.a;
                                    let val_b = val;

                                    // Conditions:
                                    // `i` (OPR) not target.
                                    // `prev_idx` (LIT b) not target.
                                    // `prev2_idx` (LIT a) CAN be target.

                                    if !targets.contains(&i) && !targets.contains(&prev_idx) {
                                        let res = match op {
                                            Operator::ADD => Some(val_a + val_b),
                                            Operator::SUB => Some(val_a - val_b),
                                            Operator::MUL => Some(val_a * val_b),
                                            Operator::DIV if val_b != 0 => Some(val_a / val_b),
                                            Operator::EQL => {
                                                Some(if val_a == val_b { 1 } else { 0 })
                                            }
                                            Operator::NEQ => {
                                                Some(if val_a != val_b { 1 } else { 0 })
                                            }
                                            Operator::LSS => {
                                                Some(if val_a < val_b { 1 } else { 0 })
                                            }
                                            Operator::LEQ => {
                                                Some(if val_a <= val_b { 1 } else { 0 })
                                            }
                                            Operator::GTR => {
                                                Some(if val_a > val_b { 1 } else { 0 })
                                            }
                                            Operator::GEQ => {
                                                Some(if val_a >= val_b { 1 } else { 0 })
                                            }
                                            _ => None,
                                        };

                                        if let Some(r) = res {
                                            new_code.pop(); // Remove LIT b
                                            new_code.pop(); // Remove LIT a
                                            // Push LIT res. Use prev2_idx as origin to keep mapping valid?
                                            // Actually, we want `prev2_idx` to map to this new instruction.
                                            new_code.push((
                                                Instruction::new(OpCode::LIT, 0, r),
                                                prev2_idx,
                                            ));
                                            changed = true;
                                            pushed = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !pushed {
            new_code.push((instr, i));
        }
        i += 1;
    }

    if !changed {
        return (code.to_vec(), false);
    }

    // 3. Build mapping
    let mut orig_to_new = vec![None; code.len()];
    for (new_idx, &(_, orig_idx)) in new_code.iter().enumerate() {
        orig_to_new[orig_idx] = Some(new_idx);
    }

    let mut map = vec![0; code.len()];
    let mut next_valid = new_code.len();
    for k in (0..code.len()).rev() {
        if let Some(idx) = orig_to_new[k] {
            map[k] = idx;
            next_valid = idx;
        } else {
            map[k] = next_valid;
        }
    }

    // 4. Fix jumps and strip indices
    let final_code: Vec<Instruction> = new_code
        .into_iter()
        .map(|(mut instr, _)| {
            if instr.f == OpCode::JMP || instr.f == OpCode::JPC {
                instr.a = map[instr.a as usize] as i64;
            }
            instr
        })
        .collect();

    (final_code, true)
}
