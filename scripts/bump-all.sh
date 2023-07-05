#!/usr/bin/env bash
set -x

# Get the latest commit message
commit_message=$(git log --format=%B -n 1)

# Check if the commit message matches the target string
if [[ "$commit_message" == "Bump npm swc-plugin-fusion" ]]; then
  echo "Last commit message is 'Bump npm swc-plugin-fusion'. Skipping the job."
  exit 0
fi

git config --local --unset-all credential.helper
git remote remove origin
git remote add origin https://${GH_TOKEN}@github.com/tajo/swc-plugin-fusion.git

echo "Setting git user.name and user.email to last commit author"
git config --global user.name "$(git log -n 1 --pretty=format:%an)"
git config --global user.email "$(git log -n 1 --pretty=format:%ae)"
(cd ./packages/fusion && npm version patch)

git status
git add .
git commit -m "Bump npm swc-plugin-fusion"
git push origin main
git push origin main --tags
npm publish --workspaces

