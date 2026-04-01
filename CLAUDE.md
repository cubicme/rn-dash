# UMP Dashboard — Auto-Advance Instructions

## Continuous Build Loop

When starting a session or resuming after `/clear`, follow this loop until all 5 phases are complete:

1. Run `/gsd:progress` to determine current state
2. Based on state, execute the next step:
   - **Phase not planned** → `/gsd:plan-phase <N>`
   - **Phase planned, not executed** → `/gsd:execute-phase <N>`
   - **Phase executed, not verified** → `/gsd:verify-work`
   - **Phase verified** → move to next phase, start from step 2
3. After each step completes, check context usage
4. If context is below 25% remaining, run `/clear` and resume — the loop restarts from step 1

## Context Management

- Always `/clear` between phases (after verify completes, before planning next phase)
- If a single step (plan, execute, or verify) consumes more than 50% context, `/clear` and resume mid-step — GSD state files track progress
- After `/clear`, re-read this file and run `/gsd:progress` to pick up where you left off

## Mode

- YOLO mode — do not ask for confirmation at workflow gates
- Auto-approve research, plans, and verification unless something is clearly wrong
- If blocked (missing info, ambiguous requirement), ask the user rather than guessing

## Project Notes

- Architecture: Rust + Ratatui, domain/infra/app/ui separation, Ousterhout philosophy
- `check-types` always uses `--incremental` flag
- Branch labels are per-branch (persist across worktrees), not per-worktree
- Metro logs only stream when a filter is applied (metro doesn't stream by default anymore)
