#!/bin/bash
# Push Proof of Emotion to GitHub
#
# Usage:
#   1. Create a new repository on GitHub: https://github.com/new
#   2. Copy the SSH or HTTPS URL (e.g., git@github.com:username/proof-of-emotion.git)
#   3. Run: ./push_to_github.sh <your-repo-url>
#
# Example:
#   ./push_to_github.sh git@github.com:chronocoder/proof-of-emotion.git

if [ -z "$1" ]; then
    echo "❌ Error: Repository URL required"
    echo ""
    echo "Usage: ./push_to_github.sh <repo-url>"
    echo ""
    echo "Example:"
    echo "  ./push_to_github.sh git@github.com:username/proof-of-emotion.git"
    echo ""
    echo "Create new repo at: https://github.com/new"
    exit 1
fi

REPO_URL="$1"

echo "🚀 Pushing Proof of Emotion to GitHub"
echo "Repository: $REPO_URL"
echo ""

# Add remote
git remote add origin "$REPO_URL" 2>/dev/null || git remote set-url origin "$REPO_URL"

# Rename branch to main (GitHub standard)
git branch -M main

# Push
echo "📤 Pushing to GitHub..."
git push -u origin main

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Successfully pushed to GitHub!"
    echo ""
    echo "View your repository at:"
    echo "${REPO_URL%.git}"
    echo ""
    echo "Next steps:"
    echo "  1. Add topics/tags on GitHub (blockchain, consensus, rust)"
    echo "  2. Enable GitHub Actions for CI/CD"
    echo "  3. Add repository description and README badges"
else
    echo ""
    echo "❌ Push failed. Common issues:"
    echo "  - Repository doesn't exist (create at: https://github.com/new)"
    echo "  - SSH key not configured (run: ssh -T git@github.com)"
    echo "  - Using HTTPS but need authentication (use SSH or GitHub CLI)"
fi
