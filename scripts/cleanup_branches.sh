#!/bin/bash
# Branch Cleanup Script for MEMORY_P Repository
# This script will close/delete all old feature branches

echo "🧹 MEMORY_P Branch Cleanup Script"
echo "================================="
echo ""
echo "This script will delete the following branches:"
echo ""

# List of branches to delete (excluding master and current PR)
BRANCHES_TO_DELETE=(
    "auto/dev-env-ffi-setup-20260204"
    "copilot/add-autonomous-mcp-integration"
    "copilot/add-smart-cli-for-devops"
    "copilot/complete-advanced-development-mcp"
    "copilot/configure-auto-push-management"
    "copilot/create-optimized-shared-memory-system"
    "copilot/improve-github-actions-workflows"
    "copilot/improve-nuclear-crawler-system"
    "copilot/optimize-devops-configurations"
    "dependabot/cargo/cargo-f6ecf5c85a"
)

# Display branches
for branch in "${BRANCHES_TO_DELETE[@]}"; do
    echo "  - $branch"
done

echo ""
echo "Total: ${#BRANCHES_TO_DELETE[@]} branches"
echo ""
echo "⚠️  IMPORTANT: This will permanently delete these branches!"
echo ""
read -p "Do you want to proceed? (yes/no): " confirmation

if [ "$confirmation" != "yes" ]; then
    echo "❌ Cancelled. No branches were deleted."
    exit 0
fi

echo ""
echo "🚀 Starting deletion process..."
echo ""

# Counter for tracking
SUCCESS_COUNT=0
FAILED_COUNT=0

# Delete each branch
for branch in "${BRANCHES_TO_DELETE[@]}"; do
    echo -n "Deleting $branch ... "

    if git push origin --delete "$branch" 2>/dev/null; then
        echo "✅ Success"
        ((SUCCESS_COUNT++))
    else
        echo "❌ Failed"
        ((FAILED_COUNT++))
    fi
done

echo ""
echo "================================="
echo "📊 Summary:"
echo "  ✅ Successfully deleted: $SUCCESS_COUNT"
echo "  ❌ Failed: $FAILED_COUNT"
echo "  📝 Total processed: ${#BRANCHES_TO_DELETE[@]}"
echo ""

if [ $SUCCESS_COUNT -eq ${#BRANCHES_TO_DELETE[@]} ]; then
    echo "✨ All branches deleted successfully!"
else
    echo "⚠️  Some branches failed to delete. Check permissions or if branches still exist."
fi

echo ""
echo "Note: The following branches were preserved:"
echo "  - master (main production branch)"
echo "  - copilot/add-advanced-memory-mcp (current PR - merge before deleting)"
