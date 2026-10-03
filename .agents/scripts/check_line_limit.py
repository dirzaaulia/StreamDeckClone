import json
import os
import re
import sys

payload = json.load(sys.stdin)
args = payload.get("toolCall", {}).get("args", {})
target = args.get("TargetFile") or ""
content = args.get("CodeContent") or ""

# Target file extension checks
is_kotlin = target.endswith(".kt")
is_rust = target.endswith(".rs")

if is_kotlin or is_rust:
    # 1. Line Limit Guard (Max 250 lines)
    line_count = len(content.splitlines()) if content else 0
    if not content and os.path.exists(target):
        with open(target, "r", encoding="utf-8", errors="ignore") as f:
            line_count = len(f.readlines())

    if line_count > 250:
        file_type = "Kotlin" if is_kotlin else "Rust"
        print(json.dumps({
            "decision": "deny",
            "reason": (
                f"BLOCKED BY LINE-GUARD: {os.path.basename(target)} has {line_count} lines (Max: 250). "
                f"You MUST decompose this {file_type} file into multiple modules before saving."
            )
        }))
        sys.exit(0)

    # 2. Hardcoded Text Guard in Presentation/UI (Kotlin only)
    if is_kotlin and any(keyword in target for keyword in ["presentation", "ui", "Screen", "Component"]):
        raw_text_match = re.search(r'\bText\(\s*"(?!\$)[A-Za-z0-9 ]{3,}"\s*\)', content)
        if raw_text_match:
            print(json.dumps({
                "decision": "deny",
                "reason": (
                    f"BLOCKED BY RESOURCE-GUARD: Detected hardcoded text literal '{raw_text_match.group(0)}' "
                    f"in {os.path.basename(target)}. Extract to strings.xml and use stringResource()."
                )
            }))
            sys.exit(0)

        # 3. Raw Hex Color in Composable Guard (Kotlin only)
        raw_color_match = re.search(r'\bColor\(0x[0-9a-fA-F]{6,8}\)', content)
        if raw_color_match:
            print(json.dumps({
                "decision": "deny",
                "reason": (
                    f"BLOCKED BY COLOR-GUARD: Detected raw Color instantiation '{raw_color_match.group(0)}' "
                    f"in {os.path.basename(target)}. Map through MaterialTheme.colorScheme or design tokens."
                )
            }))
            sys.exit(0)

# Allow tool execution
print(json.dumps({"decision": "allow"}))
