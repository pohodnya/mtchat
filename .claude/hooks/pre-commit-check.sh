#!/bin/bash
# Claude Code PreToolUse hook - validates code before git commit
# Mirrors all CI checks from .github/workflows/ci.yml

# Read input from stdin
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Only check git commit commands
if [[ "$COMMAND" =~ git[[:space:]]+commit ]]; then

  # === Rust Checks ===
  if [ -d "$CLAUDE_PROJECT_DIR/mtchat-rust" ]; then
    cd "$CLAUDE_PROJECT_DIR/mtchat-rust"

    # 1. Rust Format
    if ! cargo fmt --check 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Rust format check failed. Run: cd mtchat-rust && cargo fmt"}}'
      exit 0
    fi

    # 2. Rust Clippy (warnings as errors)
    if ! cargo clippy --quiet -- -D warnings 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Rust clippy failed. Run: cd mtchat-rust && cargo clippy -- -D warnings"}}'
      exit 0
    fi

    # 3. Rust Unit Tests
    if ! cargo test --lib --quiet 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Rust unit tests failed. Run: cd mtchat-rust && cargo test --lib"}}'
      exit 0
    fi
  fi

  # === Vue SDK Checks ===
  if [ -d "$CLAUDE_PROJECT_DIR/mtchat-vue" ]; then
    cd "$CLAUDE_PROJECT_DIR/mtchat-vue"

    # 4. Vue SDK typecheck
    if ! npm run typecheck 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Vue SDK typecheck failed. Run: cd mtchat-vue && npm run typecheck"}}'
      exit 0
    fi

    # 5. Vue SDK build
    if ! npm run build 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Vue SDK build failed. Run: cd mtchat-vue && npm run build"}}'
      exit 0
    fi
  fi

  # === Vue PrimeVue Checks ===
  if [ -d "$CLAUDE_PROJECT_DIR/mtchat-vue-primevue" ]; then
    cd "$CLAUDE_PROJECT_DIR/mtchat-vue-primevue"

    # 6. Vue PrimeVue typecheck
    if ! npm run typecheck 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Vue PrimeVue typecheck failed. Run: cd mtchat-vue-primevue && npm run typecheck"}}'
      exit 0
    fi

    # 7. Vue PrimeVue build
    if ! npm run build 2>/dev/null; then
      echo '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"❌ Vue PrimeVue build failed. Run: cd mtchat-vue-primevue && npm run build"}}'
      exit 0
    fi
  fi

fi

exit 0
