#!/usr/bin/env bash
set -x

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
