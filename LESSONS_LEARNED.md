# Lessons Learned - STOP MAKING THESE MISTAKES

## Testing Lessons

### 1. TEST THE ACTUAL FUCKING PROBLEM
- **WRONG**: Testing `--help` when the bug is about Ctrl+C during actual execution
- **RIGHT**: Test the actual running application with real video/test sources
- **Why I was stupid**: Testing --help doesn't test shutdown at all - it's a completely different code path

### 2. NEVER IGNORE TESTS
- **WRONG**: Adding `#[ignore]` to hide failing tests
- **RIGHT**: Let tests fail so we can see and fix the actual problems
- **Why I was stupid**: Ignoring tests defeats the entire purpose - we WANT to see failures

### 3. USE REALISTIC TEST DATA
- **WRONG**: Using `/tmp/test.mp4` on Windows (doesn't exist)
- **RIGHT**: Use actual files that exist or test patterns like `videotestsrc://`
- **Why I was stupid**: Windows doesn't have /tmp, and non-existent files cause different failures

## Code Review Lessons

### 4. CRITICAL BUGS FIRST
- When there are showstopper bugs (app won't exit, video won't play), those are #1 priority
- Stop suggesting new features when basic functionality is broken

### 5. UNDERSTAND THE PLATFORM
- Windows file paths: `C:\path\to\file` not `/tmp/file`
- Windows URIs: `file://C:/path` or `file:///C:/path`
- Ctrl+C handling is different on Windows vs Unix

## General Development

### 6. READ THE ERROR MESSAGES
- If the bug report says "Ctrl+C doesn't work", test THAT specific scenario
- Don't test adjacent functionality and assume it's the same

### 7. REGRESSION TESTS MUST TEST THE EXACT BUG
- If Ctrl+C shutdown was broken, the test must:
  1. Start the app normally (not --help)
  2. Send Ctrl+C signal
  3. Verify it exits cleanly
  4. Fail if it hangs

### 8. DON'T OVER-ENGINEER SIMPLE FIXES
- Stop creating complex abstractions when a simple test will do
- Stop adding unnecessary dependencies or features

## Shutdown Bug Specific

### 9. THE SHUTDOWN BUG PATTERN
- **Symptom**: App prints "Received interrupt signal, shutting down..." but doesn't exit
- **Test**: Must verify process actually terminates, not just that it prints a message
- **Regression**: This bug keeps coming back, so the test is CRITICAL

### 10. WINDOWS PROCESS TERMINATION
- GenerateConsoleCtrlEvent is unreliable across process groups
- May need to use TerminateProcess as fallback
- Test both graceful and forced termination

---

## REMEMBER: 
- Test the actual bug, not something vaguely related
- Never ignore failing tests
- Use real, valid test data for the platform
- Critical bugs before new features
- Read and understand what the user is actually asking for