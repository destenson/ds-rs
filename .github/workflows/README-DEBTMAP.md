# Debtmap Code Quality Analysis Workflow

## Overview

This GitHub Actions workflow automatically analyzes code quality using [Debtmap](https://github.com/iepathos/debtmap) to identify technical debt, code smells, and quality issues. It runs on every push, pull request, and weekly schedule.

## Features

- **Code Coverage Generation**: Uses `cargo-llvm-cov` to generate comprehensive test coverage
- **Debtmap Analysis**: Identifies technical debt patterns and code quality issues
- **PR Comments**: Automatically comments on pull requests with analysis results
- **Quality Gates**: Configurable thresholds to fail builds if quality degrades
- **Report Artifacts**: Saves HTML, JSON, and Markdown reports for review

## Workflow Triggers

- **Push**: Runs on pushes to main/master/develop branches
- **Pull Request**: Analyzes PRs before merge
- **Schedule**: Weekly analysis every Monday at 3 AM UTC
- **Manual**: Can be triggered manually via GitHub UI

## Jobs

### 1. Code Coverage (`code-coverage`)
- Installs GStreamer and system dependencies
- Generates LLVM coverage reports using `cargo-llvm-cov`
- Uploads coverage data to Codecov (optional)
- Creates coverage artifacts for Debtmap

### 2. Debtmap Analysis (`debtmap-analysis`)
- Installs and runs Debtmap
- Uses coverage data if available
- Generates multiple report formats (JSON, HTML, Markdown)
- Posts results as PR comment

### 3. Quality Gate (`quality-gate`)
- Checks if metrics meet configured thresholds
- Can block merge if quality is below standards
- Creates GitHub Actions summary

## Configuration

### Customizing Thresholds

Edit the quality gate section to add your thresholds:

```yaml
# Example threshold checks
DEBT_SCORE=$(jq '.summary.debt_score' ./debtmap-reports/debtmap-report.json)
MAX_DEBT_SCORE=7.0

if (( $(echo "$DEBT_SCORE > $MAX_DEBT_SCORE" | bc -l) )); then
  echo "❌ Technical debt score too high: $DEBT_SCORE (max: $MAX_DEBT_SCORE)"
  exit 1
fi
```

### Debtmap Configuration

Create a `.debtmap.yml` file in your repository root to configure Debtmap:

```yaml
# .debtmap.yml
analysis:
  ignore_paths:
    - target/
    - vendor/
    - "*.generated.rs"
  
  metrics:
    complexity_threshold: 10
    duplication_threshold: 50
    coverage_threshold: 80
    
  rules:
    - name: "TODO comments"
      pattern: "TODO|FIXME|HACK"
      severity: "medium"
    
    - name: "Unwrap usage"
      pattern: "\.unwrap\(\)"
      severity: "high"
      message: "Avoid using unwrap(), use proper error handling"
```

## Reports

### Available Reports

1. **coverage-reports/**
   - `lcov.info` - LCOV coverage data
   - `coverage.json` - JSON coverage data
   - `coverage-summary.json` - Coverage summary

2. **debtmap-reports/**
   - `debtmap-report.json` - Raw Debtmap analysis data
   - `debtmap-report.html` - Interactive HTML report
   - `debtmap-report.md` - Markdown report for PR comments

### Viewing Reports

- **PR Comments**: Debtmap summary posted automatically
- **Actions Summary**: View in GitHub Actions run summary
- **Artifacts**: Download full reports from Actions artifacts
- **Codecov**: If configured, view coverage on codecov.io

## Local Testing

To run Debtmap locally:

```bash
# Install debtmap
pip install debtmap

# Generate coverage (optional)
cargo llvm-cov --lcov --output-path lcov.info

# Run analysis
debtmap analyze --coverage lcov.info --output report.json

# Generate HTML report
debtmap report --input report.json --output report.html --format html

# Open report
open report.html  # macOS
xdg-open report.html  # Linux
start report.html  # Windows
```

## Troubleshooting

### Coverage Generation Fails

If coverage generation fails, ensure you have:
- `llvm-tools-preview` component installed
- All system dependencies (GStreamer, Cairo, etc.)
- Sufficient permissions for instrumented binaries

### Debtmap Not Finding Issues

Check that:
- Your `.debtmap.yml` configuration is valid
- Source files are not in ignored paths
- Coverage data is being generated correctly

### PR Comments Not Appearing

Ensure:
- GitHub token has comment permissions
- PR is from same repository (not a fork)
- Workflow has write permissions

## Benefits

1. **Automated Quality Tracking**: Continuous monitoring of code health
2. **Early Detection**: Catch quality issues before merge
3. **Trend Analysis**: Track quality improvements/degradation over time
4. **Team Awareness**: Automatic PR comments keep team informed
5. **Objective Metrics**: Data-driven quality decisions

## Integration with Other Tools

This workflow can be extended to integrate with:
- **SonarQube**: Upload analysis results
- **Code Climate**: Send quality metrics
- **Slack/Discord**: Notify team of quality issues
- **Jira**: Create tickets for high-severity issues

## Customization Examples

### Add Slack Notification

```yaml
- name: Notify Slack on failure
  if: failure()
  uses: slackapi/slack-github-action@v1
  with:
    webhook-url: ${{ secrets.SLACK_WEBHOOK }}
    payload: |
      {
        "text": "Code quality check failed!",
        "attachments": [{
          "color": "danger",
          "title": "Debtmap Analysis Failed",
          "text": "Technical debt exceeds threshold"
        }]
      }
```

### Generate Badge

```yaml
- name: Generate quality badge
  run: |
    SCORE=$(jq '.summary.quality_score' debtmap-report.json)
    curl -X POST https://img.shields.io/badge/quality-${SCORE}-green
```

## Resources

- [Debtmap Documentation](https://github.com/iepathos/debtmap)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [GitHub Actions Documentation](https://docs.github.com/actions)
- [Codecov Documentation](https://docs.codecov.io)