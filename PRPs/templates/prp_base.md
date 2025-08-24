# PRP: [Title - Clear, Concise Description of the Objective]

## Executive Summary

[2-3 sentences describing WHAT this PRP aims to achieve and WHY it matters to the project]

## Problem Statement

### Current State
[Describe the current situation - what exists now, what's missing, what's broken]

### Desired State
[Describe where we want to be after this PRP is implemented]

### Business Value
[Why does this matter? What value does it provide to users/stakeholders?]

## Requirements

### Functional Requirements
[Numbered list of WHAT the system must do, not HOW]

1. **[Requirement Name]**: [Description of the capability needed]
2. **[Requirement Name]**: [Description of the capability needed]
3. ...

### Non-Functional Requirements
[Performance, security, scalability, maintainability requirements]

1. **Performance**: [e.g., "Must handle X requests per second"]
2. **Reliability**: [e.g., "Must maintain 99.9% uptime"]
3. **Security**: [e.g., "Must validate all user inputs"]
4. ...

### Context and Research
[Relevant background information, research findings, or context that informs the PRP]

### Documentation & References (list all context needed to implement the feature)
```yaml
# MUST READ - Include these in your context window
- url: [Official API docs URL]
  why: [Specific sections/methods you'll need]
  
- file: [path/to/example.py]
  why: [Pattern to follow, gotchas to avoid]
  
- doc: [Library documentation URL] 
  section: [Specific section about common pitfalls]
  critical: [Key insight that prevents common errors]

- docfile: [PRPs/ai_docs/file.md]
  why: [docs that the user has pasted in to the project]

```

### list of tasks to be completed to fullfill the PRP in the order they should be completed

```yaml
Task 1:
MODIFY src/existing_module.py:
  - FIND pattern: "class OldImplementation"
  - INJECT after line containing "def __init__"
  - PRESERVE existing method signatures

CREATE src/new_feature.py:
  - MIRROR pattern from: src/similar_feature.py
  - MODIFY class name and core logic
  - KEEP error handling pattern identical

...(...)

Task N:
...
```

### Out of Scope
[Explicitly list what this PRP does NOT cover]

- [Thing that won't be addressed]
- [Another thing that's excluded]

## Success Criteria

[Measurable criteria to determine if the PRP has been successfully implemented]

- [ ] [Specific, measurable outcome]
- [ ] [Another measurable outcome]
- [ ] [User-facing or technical validation]

## Dependencies

### Technical Dependencies
- [External library or service required]
- [Existing system that must be in place]

### Knowledge Dependencies
- [Documentation that must be understood]
- [Expertise required]

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| [Risk description] | Low/Medium/High | Low/Medium/High | [How to handle it] |
| [Another risk] | Low/Medium/High | Low/Medium/High | [How to handle it] |

## Architecture Decisions

### Decision: [Title of architectural choice]
**Options Considered:**
1. [Option A]
2. [Option B]

**Decision:** [Which option and why - focus on trade-offs and rationale]

**Rationale:** [Why this choice best serves the requirements]

## Validation Strategy

[How will we verify that the implementation meets the requirements?]

- **Unit Testing**: [What aspects need unit tests]
- **Integration Testing**: [What integrations need testing]
- **User Acceptance**: [How users will validate the solution]

## Future Considerations

[What might come next after this PRP? What doors does this open?]

- [Potential future enhancement]
- [Possible follow-up work]

## References

[Links to relevant documentation, standards, or specifications - but NOT implementation examples]

- [Relevant specification or standard]
- [Domain documentation]
- [Business requirements document]

---

## PRP Metadata

- **Author**: [Name]
- **Created**: [Date]
- **Last Modified**: [Date]
- **Status**: Draft | In Review | Approved | Implemented
- **Confidence Level**: [1-10] - [Rationale for confidence score]
