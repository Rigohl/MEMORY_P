#!/bin/bash
# GitLab CI/CD Cleanup Job - Executed by pipeline
# This job removes redundant documentation files after consolidation

set -e

echo "════════════════════════════════════════════════════════════════"
echo "  MEMORY_P PHASE 4 CLEANUP - Executed by GitLab CI/CD Pipeline"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Files to delete (consolidated into FFI_AND_DEVOPS_GUIDE.md)
declare -a FILES_TO_DELETE=(
    "FFI_AUDIT_REAL_STATE.md"
    "FFI_PHASE_1_COMPLETION.md"
    "FFI_REAL_IMPLEMENTATION_STATUS.md"
    "GITLAB_IMPROVEMENTS_2026.md"
    "GITLAB_PHASE_1_IMPLEMENTATION.md"
    "PHASE_3_STATUS.md"
    "ROOT_FUSION_SUMMARY.md"
    "QUICK_FINISH.md"
    "build_errors_full.txt"
    "check_output_full.txt"
    "check_output.txt"
    "full_check.txt"
)

echo "📋 Files to delete:"
for file in "${FILES_TO_DELETE[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✓ $file (will delete)"
    fi
done
echo ""

# Perform deletion
echo "🗑️  Deleting redundant files..."
for file in "${FILES_TO_DELETE[@]}"; do
    if [ -f "$file" ]; then
        rm -f "$file"
        echo "  ✓ Deleted: $file"
    fi
done
echo ""

# Verify deletion
echo "✅ Cleanup complete!"
echo ""
echo "📊 Repository state after cleanup:"
du -sh . | awk '{print "  Total: " $1}'
find . -maxdepth 1 -type f \( -name "*.md" -o -name "*.txt" \) ! -path "./.git/*" | wc -l | awk '{print "  Root docs: " $1 " files"}'
echo ""
