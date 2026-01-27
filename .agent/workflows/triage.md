---
description: Procedure for formalizing, categorizing, and creating new production-grade issues.
---

1. **Deduplication Check**
   - Run `gr list --format json` and filter for keywords related to the new issue.
   - **Stop** if a similar issue already exists; update the existing one instead.

2. **Classification**
   - **Type**: `task` (general work), `bug` (regression/error), `feature` (new capability), or `epic` (parent container).
   - **Priority**: 1 (Critical), 2 (High), 3 (Medium), 4 (Low), 5 (De-prioritized).

3. **Formal Creation**
   - Execute `gr create "<TITLE>" -d "<DESCRIPTION>" -p <1-5> -t <TYPE>`.
   - Take note of the newly generated `ID` (e.g., `gr-xxxxxx`).

4. **Relationship Mapping**
   - If this is part of a larger effort, link it to a parent Epic:
     `gr update --id <PARENT_EPIC_ID> --add-dependency <NEW_ISSUE_ID>`
   - Alternatively, add it as a primary dependency for another issue.

5. **Success**
   - Run `gr show <ID>` to verify metadata. The issue is now ready for `/discovery-workflow`.
